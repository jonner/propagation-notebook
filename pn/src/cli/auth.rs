use toasty::Db;

use crate::cli::OutputFormat;

mod permissions;
mod roles;
mod users;

#[derive(Debug, clap::Subcommand)]
pub enum AuthCommands {
    #[command(about = "Manage users")]
    Users {
        #[command(subcommand)]
        command: users::UserCommands,
    },
    #[command(about = "Manage roles")]
    Roles {
        #[command(subcommand)]
        command: roles::RoleCommands,
    },
    #[command(about = "Manage Permissions")]
    Permissions {
        #[command(subcommand)]
        command: permissions::PermissionCommands,
    },
}
impl AuthCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            AuthCommands::Users { command } => command.run(db, format).await?,
            AuthCommands::Roles { command } => command.run(db, format).await?,
            AuthCommands::Permissions { command } => command.run(db, format).await?,
        }
        Ok(())
    }
}
