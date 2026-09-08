use crate::{
    cli::OutputFormat,
    util::dialog::{self, confirm},
    views::{
        JsonView, YamlView,
        auth::{UserDetailsView, UserListView},
    },
};

use anyhow::anyhow;
use libpropagation::auth::{
    Role, User, UserRole,
    dto::{UserCompact, UserDetails},
    hash_password,
};
use toasty::Db;
use uuid::Uuid;

#[derive(Debug, clap::Subcommand)]
pub enum UserCommands {
    #[command(about = "Print a list of users", alias = "ls")]
    List,
    #[command(about = "Show detailed information about a user")]
    Show { id: Uuid },
    #[command(about = "Add a new user to the database", alias = "new")]
    Add {
        #[arg(help = "A username for the user")]
        username: String,
    },
    #[command(about = "Change the password for a user")]
    ChangePassword {
        #[arg(help = "A region ID")]
        id: Uuid,
    },
    #[command(about = "Remove a user from the database")]
    Remove {
        #[arg(help = "A user ID")]
        id: Uuid,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    AddRole {
        #[arg(help = "A user ID")]
        id: Uuid,
        #[arg(short, long, help = "A role ID")]
        role: Uuid,
    },
    RemoveRole {
        #[arg(help = "A user ID")]
        id: Uuid,
        #[arg(short, long, help = "A role ID")]
        role: Uuid,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl UserCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            UserCommands::List => {
                let users: Vec<UserCompact> = User::all()
                    .exec(db)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect();
                let output = match format {
                    OutputFormat::Text => UserListView::new(&users).render()?,
                    OutputFormat::Json => JsonView::new(&users).render()?,
                    OutputFormat::Yaml => YamlView::new(&users).render()?,
                };
                println!("{output}");
            }
            UserCommands::Show { id } => {
                load_and_display_user(id, db, format).await?;
            }
            UserCommands::Add { username } => {
                let password = dialog::password("Enter password:").run()?;
                let confirm = dialog::password("Re-enter password:").run()?;
                if password != confirm {
                    return Err(anyhow!("Passwords do not match"));
                }
                let pwhash = hash_password(&password)?;
                let user = User::create()
                    .username(username)
                    .pwhash(pwhash)
                    .exec(db)
                    .await?;

                load_and_display_user(&user.id, db, format).await?;
            }
            UserCommands::ChangePassword { id } => {
                // just validate the user id
                let _user = User::get_by_id(db, id).await?;
                let password = dialog::password("Enter new password:").run()?;
                let confirm = dialog::password("Re-enter new password:").run()?;
                if password != confirm {
                    return Err(anyhow!("Passwords do not match"));
                }
                let pwhash = hash_password(&password)?;
                User::update_by_id(id).pwhash(pwhash).exec(db).await?;
                println!("Updated");
            }
            UserCommands::Remove { id, assumeyes } => {
                if *assumeyes || {
                    load_and_display_user(id, db, format).await?;
                    confirm("Are you sure you wish to remove this user?")
                        .selected(false)
                        .run()?
                } {
                    User::delete_by_id(db, id).await?;
                }
            }
            UserCommands::AddRole { id, role } => {
                let role = Role::get_by_id(db, role)
                    .await
                    .map_err(|_| anyhow!("Role {role} not found"))?;
                UserRole::create().user_id(id).role(role).exec(db).await?;
                load_and_display_user(id, db, format).await?;
            }
            UserCommands::RemoveRole {
                id,
                role,
                assumeyes,
            } => {
                if *assumeyes || {
                    let user = User::get_by_id(db, id).await?;
                    let role = Role::get_by_id(db, role).await?;
                    confirm(&format!(
                        "Are you sure you wish to remove role '{}' from user '{}'?",
                        role.name, user.username,
                    ))
                    .selected(false)
                    .run()?
                } {
                    UserRole::delete_by_user_id_and_role_id(db, id, role).await?;
                    load_and_display_user(id, db, format).await?;
                }
            }
        }
        Ok(())
    }
}

async fn load_and_display_user(
    id: &Uuid,
    db: &mut Db,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let user: UserDetails = User::filter_by_id(id)
        .include(User::fields().roles())
        .one()
        .exec(db)
        .await?
        .into();
    let output = match format {
        OutputFormat::Text => UserDetailsView::new(&user).render()?,
        OutputFormat::Json => JsonView::new(&user).render()?,
        OutputFormat::Yaml => YamlView::new(&user).render()?,
    };
    println!("{output}");
    Ok(())
}
