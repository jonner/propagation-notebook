use std::collections::HashMap;

use anyhow::anyhow;
use clap::Parser;
use directories::ProjectDirs;
use propagation_notebook::{
    propagation::{Protocol, ProtocolType},
    region::RegionalTaxonStatus,
    taxonomy::Taxon,
};
use serde::Deserialize;
use tabled::builder::Builder as TableBuilder;
use toasty::Db;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{MainCommand, Options, propagation::PropagationCommands};

mod cli;

mod style {
    use tabled::{
        grid::{
            config::ColoredConfig,
            dimension::CompleteDimension,
            records::{ExactRecords, Records},
        },
        settings::{Alignment, Modify, Style, TableOption, object::Columns},
    };

    pub struct BasicTable;

    impl<R> TableOption<R, ColoredConfig, CompleteDimension> for BasicTable
    where
        R: Records,
    {
        fn change(
            self,
            records: &mut R,
            cfg: &mut ColoredConfig,
            dimension: &mut CompleteDimension,
        ) {
            Style::empty().change(records, cfg, dimension);
        }
    }
    pub struct DetailTable;
    impl<R> TableOption<R, ColoredConfig, CompleteDimension> for DetailTable
    where
        R: ExactRecords + Records,
    {
        fn change(
            self,
            records: &mut R,
            cfg: &mut ColoredConfig,
            dimension: &mut CompleteDimension,
        ) {
            BasicTable.change(records, cfg, dimension);
            Modify::new(Columns::first())
                .with(Alignment::right())
                .change(records, cfg, dimension);
        }
    }
}

fn truncate_with_summary(s: &str, max_chars: usize) -> String {
    let extra_chars = s.chars().count().saturating_sub(max_chars);
    if extra_chars == 0 {
        return s.to_string();
    }
    s.chars().take(max_chars).collect::<String>() + &format!("... [{extra_chars} more characters]")
}

fn join_or_default<T, F>(items: &[T], default: &str, extract: F) -> String
where
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        default.to_string()
    } else {
        items.iter().map(extract).collect::<Vec<_>>().join("\n")
    }
}

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

    let mut tbuilder = TableBuilder::default();
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
        MainCommand::Propagation { command } => match command {
            PropagationCommands::List { r#type } => {
                let mut query = Protocol::all();
                if let Some(t) = r#type {
                    query = query.filter(Protocol::fields().r#type().eq(t));
                }
                let protocols = query.exec(&mut db).await?;
                let mut tbuilder = TableBuilder::default();
                tbuilder.push_record(["ID", "Name", "Type"]);
                for protocol in protocols {
                    tbuilder.push_record([
                        protocol.id.to_string(),
                        protocol.name,
                        protocol.r#type.to_string(),
                    ])
                }
                println!("{}", tbuilder.build().with(style::BasicTable));
            }
            PropagationCommands::Show { id } => {
                let p = Protocol::filter_by_id(id).one().exec(&mut db).await?;
                let mut tbuilder = TableBuilder::default();
                tbuilder.push_record(["ID", &p.id.to_string()]);
                tbuilder.push_record(["Name", &p.name]);
                tbuilder.push_record(["Type", &p.r#type.to_string()]);
                tbuilder.push_record(["Notes", &p.notes.unwrap_or_else(|| "-".into())]);
                tbuilder.push_record(["Instructions", &p.instructions]);
                println!("{}", tbuilder.build().with(style::DetailTable));
            }
            PropagationCommands::Add {
                name,
                r#type,
                notes,
            } => {
                let item = Protocol::create()
                    .name(name)
                    .r#type(r#type)
                    .notes(notes)
                    .exec(&mut db)
                    .await?;
                println!("Added protocol {}", item.id);
            }
            PropagationCommands::Modify {
                id,
                name,
                r#type,
                notes,
            } => {
                let mut query = Protocol::update_by_id(id);
                if let Some(name) = name {
                    query = query.name(name);
                }
                if let Some(t) = r#type {
                    query = query.r#type(t);
                }

                if let Some(notes) = notes {
                    query = query.notes(notes);
                }
                query.exec(&mut db).await?;
                println!("Updated protocol {id}");
            }
            PropagationCommands::Remove { id, assumeyes } => {
                if assumeyes
                    || inquire::Confirm::new(
                        "Are you sure you wish to remove this Propagation protocol?",
                    )
                    .with_default(false)
                    .with_help_message("It will remove all related steps")
                    .prompt()?
                {
                    Protocol::delete_by_id(&mut db, id).await?;
                    println!("Removed propagation protocol {id}");
                }
            }
            PropagationCommands::Import { path } => {
                #[derive(Debug, Deserialize)]
                struct ProtocolInfo {
                    pub name: String,
                    pub instructions: String,
                    pub notes: Option<String>,
                    pub r#type: ProtocolType,
                }
                let protocols: Vec<ProtocolInfo> =
                    serde_yaml::from_reader(std::fs::File::open(path)?)?;
                for p in protocols {
                    Protocol::create()
                        .name(p.name)
                        .instructions(p.instructions)
                        .notes(p.notes)
                        .r#type(p.r#type)
                        .exec(&mut db)
                        .await?;
                }
            }
        },
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
