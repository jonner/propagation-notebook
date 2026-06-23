use anyhow::anyhow;
use clap::Parser;
use directories::ProjectDirs;
use toasty::Db;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{MainCommand, Options};

mod cli;
mod style;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::default().add_directive(LevelFilter::WARN.into()));
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();
    let project_dir = ProjectDirs::from("org", "quotidian", "propagation-notebook")
        .ok_or_else(|| anyhow!("Unable to determine project data directory"))?
        .data_dir()
        .to_path_buf();
    std::fs::create_dir_all(&project_dir)?;
    let options = Options::parse();
    let db_uri = match std::env::var("PN_DB_URI") {
        Ok(s) => Ok(s),
        Err(std::env::VarError::NotPresent) => Ok(format!(
            "sqlite:{}",
            project_dir
                .join("propagation-notebook.sqlite")
                .to_str()
                .unwrap()
        )),
        e => e,
    }?;
    let mut db = Db::builder()
        .models(libpropagation::models())
        .connect(&db_uri)
        .await?;
    match options.command {
        MainCommand::Init => init_command(&mut db).await?,
        MainCommand::Taxa { command } => command.run(&mut db).await?,
        MainCommand::Regions { command } => command.run(&mut db).await?,
        MainCommand::Cleaning { command } => command.run(&mut db).await?,
        MainCommand::Propagation { command } => command.run(&mut db).await?,
    };
    Ok(())
}

async fn init_command(db: &mut Db) -> anyhow::Result<()> {
    match db.push_schema().await {
        Ok(()) => Ok(()),
        Err(e) => {
            if e.is_driver_operation_failed() {
                // this is the error that occurs when we try to push a schema
                // after the schema has already been applied
                Ok(())
            } else {
                // Report other errors
                Err(e.into())
            }
        }
    }
}
