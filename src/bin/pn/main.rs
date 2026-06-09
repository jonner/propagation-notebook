use std::collections::HashMap;

use anyhow::anyhow;
use clap::Parser;
use directories::ProjectDirs;
use propagation_notebook::{region::RegionalTaxonStatus, taxonomy::Taxon};
use toasty::Db;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{MainCommand, Options};

mod cli;
mod style;
mod util;

async fn list_regional_taxa(db: &mut toasty::Db, region_id: u64) -> anyhow::Result<()> {
    let regional_statuses =
        RegionalTaxonStatus::filter(RegionalTaxonStatus::fields().region_id().eq(region_id))
            // FIXME: We want to order by a taxon sequence, but
            // toasty doesn't yet support ordering by data in a relation
            .exec(db)
            .await?;

    // FIXME: it's too slow to include all relations, so query the taxa separately
    let taxa = Taxon::filter(
        Taxon::fields().id().in_list(
            regional_statuses
                .iter()
                .map(|s| s.taxon_id)
                .collect::<Vec<_>>(),
        ),
    )
    .order_by(Taxon::fields().sequence().asc())
    .exec(db)
    .await?;

    // since we can't order the regional status list by taxon
    // sequence, we need to iterate through the sorted taxon list, and then look up the
    // regional status from a hash table
    let map = regional_statuses
        .into_iter()
        .map(|s| (s.taxon_id, s))
        .collect::<HashMap<_, _>>();

    let mut tbuilder = tabled::builder::Builder::default();
    tbuilder.push_record([
        "ID",
        "Taxon",
        "Origin",
        "Status",
        "C-value",
        "Wetland Indicator",
    ]);
    for taxon in taxa {
        let status = map.get(&taxon.id).unwrap();
        tbuilder.push_record([
            taxon.id.to_string(),
            taxon.complete_name,
            status
                .origin
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
            status
                .conservation_status
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
            status
                .c_value
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
            status
                .wetland_indicator
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
        ]);
    }
    println!("{}", tbuilder.build().with(style::BasicTable));
    Ok(())
}

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
        .models(propagation_notebook::models())
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
