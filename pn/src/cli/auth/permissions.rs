use crate::{
    cli::OutputFormat,
    util::dialog::confirm,
    views::{
        JsonView, YamlView,
        auth::{PermissionDetailsView, PermissionListView},
    },
};

use libpropagation::auth::{
    Permission, PermissionCode,
    dto::{PermissionCompact, PermissionDetails},
};

use toasty::Db;
use uuid::Uuid;

#[derive(Debug, clap::Subcommand)]
pub enum PermissionCommands {
    #[command(about = "Print a list of permissions", alias = "ls")]
    List,
    #[command(about = "Show detailed information about a permission")]
    Show { id: Uuid },
    #[command(about = "Add a new permission to the database", alias = "new")]
    Add {
        #[arg(long, short, value_enum, help = "A code for the permission")]
        code: PermissionCode,
        #[arg(long, short, help = "A description of the permission")]
        description: String,
    },
    #[command(about = "Remove a permission from the database")]
    Remove {
        #[arg(help = "A permission ID")]
        id: Uuid,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl PermissionCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            PermissionCommands::List => {
                let permissions: Vec<PermissionCompact> = Permission::all()
                    .exec(db)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect();
                let output = match format {
                    OutputFormat::Text => PermissionListView::new(&permissions).render()?,
                    OutputFormat::Json => JsonView::new(&permissions).render()?,
                    OutputFormat::Yaml => YamlView::new(&permissions).render()?,
                };
                println!("{output}");
            }
            PermissionCommands::Show { id } => {
                load_and_display_permission(id, db, format).await?;
            }
            PermissionCommands::Add { code, description } => {
                tracing::debug!(?code, ?description);
                let permission: PermissionDetails = Permission::create()
                    .code(code)
                    .description(description)
                    .exec(db)
                    .await?
                    .into();

                let output = match format {
                    OutputFormat::Text => PermissionDetailsView::new(&permission).render()?,
                    OutputFormat::Json => JsonView::new(&permission).render()?,
                    OutputFormat::Yaml => YamlView::new(&permission).render()?,
                };
                println!("{output}");
            }
            PermissionCommands::Remove { id, assumeyes } => {
                if *assumeyes || {
                    load_and_display_permission(id, db, format).await?;
                    confirm("Are you sure you wish to remove this permission?")
                        .selected(false)
                        .run()?
                } {
                    Permission::delete_by_id(db, id).await?;
                }
            }
        }
        Ok(())
    }
}

async fn load_and_display_permission(
    id: &Uuid,
    db: &mut Db,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let permission: PermissionDetails = Permission::get_by_id(db, id).await?.into();
    let output = match format {
        OutputFormat::Text => PermissionDetailsView::new(&permission).render()?,
        OutputFormat::Json => JsonView::new(&permission).render()?,
        OutputFormat::Yaml => YamlView::new(&permission).render()?,
    };
    println!("{output}");
    Ok(())
}
