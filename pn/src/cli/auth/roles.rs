use crate::{
    cli::OutputFormat,
    util::{dialog::confirm, enum_to_string},
    views::{
        JsonView, YamlView,
        auth::{RoleDetailsView, RoleListView},
    },
};

use anyhow::anyhow;
use libpropagation::auth::{
    Permission, Role, RolePermission, User, UserRole,
    dto::{RoleCompact, RoleDetails},
};
use toasty::Db;
use uuid::Uuid;

#[derive(Debug, clap::Subcommand)]
pub enum RoleCommands {
    #[command(about = "Print a list of roles", alias = "ls")]
    List,
    #[command(about = "Show detailed information about a role")]
    Show { id: Uuid },
    #[command(about = "Add a new role to the database", alias = "new")]
    Add {
        #[arg(help = "A name for the role")]
        name: String,
    },
    #[command(about = "Remove a role from the database")]
    Remove {
        #[arg(help = "A role ID")]
        id: Uuid,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    AddPermission {
        #[arg(help = "A user ID")]
        id: Uuid,
        #[arg(short, long, help = "A permission ID")]
        permission: Uuid,
    },
    RemovePermission {
        #[arg(help = "A user ID")]
        id: Uuid,
        #[arg(short, long, help = "A permission ID")]
        permission: Uuid,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl RoleCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            RoleCommands::List => {
                let roles: Vec<RoleCompact> = Role::all()
                    .exec(db)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect();
                let output = match format {
                    OutputFormat::Text => RoleListView::new(&roles).render()?,
                    OutputFormat::Json => JsonView::new(&roles).render()?,
                    OutputFormat::Yaml => YamlView::new(&roles).render()?,
                };
                println!("{output}");
            }
            RoleCommands::Show { id } => {
                load_and_display_role(id, db, format).await?;
            }
            RoleCommands::Add { name } => {
                let role = Role::create().name(name).exec(db).await?;
                load_and_display_role(&role.id, db, format).await?;
            }
            RoleCommands::Remove { id, assumeyes } => {
                if *assumeyes || {
                    load_and_display_role(id, db, format).await?;
                    let nusers = User::filter(
                        User::fields()
                            .user_roles()
                            .any(UserRole::fields().role_id().eq(id)),
                    )
                    .count()
                    .exec(db)
                    .await?;
                    confirm(&format!(
                        "Are you sure you wish to remove this role? It is used by {} users",
                        nusers
                    ))
                    .selected(false)
                    .run()?
                } {
                    Role::delete_by_id(db, id).await?;
                    load_and_display_role(id, db, format).await?;
                }
            }
            RoleCommands::AddPermission { id, permission } => {
                let permission = Permission::get_by_id(db, permission)
                    .await
                    .map_err(|_| anyhow!("Permission {permission} not found"))?;
                RolePermission::create()
                    .role_id(id)
                    .permission(permission)
                    .exec(db)
                    .await?;
                load_and_display_role(id, db, format).await?;
            }
            RoleCommands::RemovePermission {
                id,
                permission,
                assumeyes,
            } => {
                if *assumeyes || {
                    let role = Role::get_by_id(db, id).await?;
                    let permission = Permission::get_by_id(db, permission).await?;
                    confirm(&format!(
                        "Are you sure you wish to remove permission '{}' from role '{}'?",
                        enum_to_string(&permission.code),
                        role.name,
                    ))
                    .selected(false)
                    .run()?
                } {
                    RolePermission::delete_by_role_id_and_permission_id(db, id, permission).await?;
                    load_and_display_role(id, db, format).await?;
                }
            }
        }
        Ok(())
    }
}

async fn load_and_display_role(
    id: &Uuid,
    db: &mut Db,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let role: RoleDetails = Role::filter_by_id(id)
        .include(Role::fields().permissions())
        .one()
        .exec(db)
        .await?
        .into();
    let output = match format {
        OutputFormat::Text => RoleDetailsView::new(&role).render()?,
        OutputFormat::Json => JsonView::new(&role).render()?,
        OutputFormat::Yaml => YamlView::new(&role).render()?,
    };
    println!("{output}");
    Ok(())
}
