use clap::Parser;
use toasty::Db;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{MainCommand, Options};

mod cli;
mod style;
mod util;
mod views;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::default());
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();
    let options = Options::parse();
    let mut db = libpropagation::db().await?;
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
