use std::collections::HashMap;

use anyhow::anyhow;
use clap::Parser;
use directories::ProjectDirs;
use propagation_notebook::{
    collecting::CleaningProcedure,
    propagation::{Protocol, ProtocolType},
    region::{Region, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use serde::Deserialize;
use tabled::builder::Builder as TableBuilder;
use toasty::Db;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    cli::{
        MainCommand, Options,
        cleaning::CleaningCommands,
        propagation::PropagationCommands,
        region::{RegionCommands, RegionTaxaCommands},
    },
    import_region::import_region,
};

mod cli;
mod import_region;

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
        MainCommand::Regions { command } => match command {
            RegionCommands::List => {
                let regions = Region::all()
                    .include(Region::fields().taxon_statuses())
                    .exec(&mut db)
                    .await?;
                if regions.is_empty() {
                    println!("No Regions found");
                } else {
                    let mut tbuilder = TableBuilder::default();
                    tbuilder.push_record(["ID", "Name", "Taxa"]);
                    for region in regions {
                        tbuilder.push_record([
                            region.id.to_string(),
                            region.name,
                            region.taxon_statuses.get().len().to_string(),
                        ])
                    }
                    println!("{}", tbuilder.build().with(style::BasicTable));
                }
            }
            RegionCommands::Show { id } => {
                let region = Region::filter_by_id(id)
                    .include(Region::fields().taxon_statuses())
                    .one()
                    .exec(&mut db)
                    .await?;
                let mut tbuilder = TableBuilder::default();
                tbuilder.push_record(["ID", &region.id.to_string()]);
                tbuilder.push_record(["Name", &region.name]);
                tbuilder.push_record(["Notes", &region.notes.unwrap_or_else(|| "-".to_string())]);
                tbuilder.push_record(["Taxa", &region.taxon_statuses.get().len().to_string()]);
                tbuilder.push_record([
                    "Bounds",
                    &truncate_with_summary(&region.bounds.unwrap_or_else(|| "-".to_string()), 500),
                ]);
                println!("{}", tbuilder.build().with(style::DetailTable))
            }
            RegionCommands::Modify {
                id,
                bounds,
                name,
                notes,
            } => {
                let mut update_query = Region::update_by_id(id);
                let bounds = bounds.resolve().await?;
                if let Some(name) = name {
                    update_query = update_query.name(name);
                }
                if let Some(bounds) = bounds {
                    update_query = update_query.bounds(bounds);
                }
                if let Some(notes) = notes {
                    update_query = update_query.notes(notes);
                }
                update_query.exec(&mut db).await?;
                println!("Region {id} updated");
            }
            RegionCommands::Add {
                region_name,
                bounds,
                notes,
            } => {
                let bounds = bounds.resolve().await?;
                let new_region = Region::create()
                    .name(region_name)
                    .bounds(bounds)
                    .notes(notes)
                    .exec(&mut db)
                    .await?;
                println!("Added new region {}", new_region.reference());
            }
            RegionCommands::Import { path } => {
                import_region(&mut db, path).await?;
            }
            RegionCommands::Remove { id, assumeyes } => {
                if assumeyes
                    || inquire::Confirm::new("Are you sure you wish to delete this region?")
                        .with_default(false)
                        .with_help_message("All associated data will be deleted")
                        .prompt()?
                {
                    Region::delete_by_id(&mut db, id).await?;
                    println!("Deleted region {id} from the database");
                }
            }
            RegionCommands::Taxa { region_id, command } => match command {
                RegionTaxaCommands::Show { taxon_id } => {
                    let status =
                        RegionalTaxonStatus::filter_by_taxon_id_and_region_id(taxon_id, region_id)
                            .include(RegionalTaxonStatus::fields().region())
                            .include(RegionalTaxonStatus::fields().taxon())
                            .one()
                            .exec(&mut db)
                            .await?;
                    let mut tbuilder = TableBuilder::default();
                    tbuilder.push_record(["Taxon", &status.taxon.get().reference()]);
                    tbuilder.push_record(["Region", &status.region.get().reference()]);
                    tbuilder.push_record([
                        "Origin",
                        &status
                            .origin
                            .unwrap_or(propagation_notebook::region::Origin::Unknown)
                            .to_string(),
                    ]);
                    tbuilder.push_record([
                        "C-value",
                        &status
                            .c_value
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "-".into()),
                    ]);
                    tbuilder.push_record([
                        "Conservation Status",
                        &status
                            .conservation_status
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "-".into()),
                    ]);
                    tbuilder.push_record([
                        "Wetland Indicator",
                        &status
                            .wetland_indicator
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "-".into()),
                    ]);
                    let window_str = match (status.window_start, status.window_end) {
                        (None, None) => "-".into(),
                        _ => format!(
                            "{} - {}",
                            status
                                .window_start
                                .map(|d| d.strftime("%b %d").to_string())
                                .unwrap_or("?".to_string()),
                            status
                                .window_end
                                .map(|d| d.strftime("%b %d").to_string())
                                .unwrap_or("?".to_string())
                        ),
                    };
                    tbuilder.push_record(["Harvest Window", &window_str]);
                    println!("{}", tbuilder.build().with(style::DetailTable));
                    println!();
                }
                RegionTaxaCommands::Add { taxon_id, props } => {
                    // make sure region exists
                    let _r = Region::get_by_id(&mut db, region_id).await?;
                    let s = RegionalTaxonStatus::create()
                        .region_id(region_id)
                        .taxon_id(taxon_id)
                        .origin(props.origin)
                        .c_value(props.c_value)
                        .conservation_status(props.conservation_status)
                        .wetland_indicator(props.wetland_indicator)
                        .window_start(
                            props
                                .harvest_start
                                .map(|d| d.with().year(2000).build().unwrap()),
                        )
                        .window_end(
                            props
                                .harvest_end
                                .map(|d| d.with().year(2000).build().unwrap()),
                        )
                        .exec(&mut db)
                        .await?;
                    println!("Added regional taxon {}", s.id);
                }
                RegionTaxaCommands::Modify { taxon_id, props } => {
                    // make sure region exists
                    let _r = Region::get_by_id(&mut db, region_id).await?;
                    let mut query =
                        RegionalTaxonStatus::update_by_taxon_id_and_region_id(taxon_id, region_id);
                    if let Some(origin) = props.origin {
                        query = query.origin(origin);
                    }
                    if let Some(c_value) = props.c_value {
                        query = query.c_value(c_value);
                    }
                    if let Some(conservation_status) = props.conservation_status {
                        query = query.conservation_status(conservation_status);
                    }
                    if let Some(wetland_indicator) = props.wetland_indicator {
                        query = query.wetland_indicator(wetland_indicator);
                    }
                    if let Some(harvest_start) = props.harvest_start {
                        query =
                            query.window_start(harvest_start.with().year(2000).build().unwrap());
                    }
                    if let Some(harvest_end) = props.harvest_end {
                        query = query.window_end(harvest_end.with().year(2000).build().unwrap());
                    }
                    query.exec(&mut db).await?;
                    println!("Modified taxon {} in region {}", taxon_id, region_id);
                }
                RegionTaxaCommands::List => list_regional_taxa(&mut db, region_id).await?,
                RegionTaxaCommands::Remove {
                    taxon_id,
                    assumeyes,
                } => {
                    if assumeyes
                        || inquire::Confirm::new(
                            "Are you sure you wish to remove this regional taxon?",
                        )
                        .with_default(false)
                        .prompt()?
                    {
                        RegionalTaxonStatus::delete_by_taxon_id_and_region_id(
                            &mut db, taxon_id, region_id,
                        )
                        .await?;
                        println!("Removed taxon {} from region {}", taxon_id, region_id);
                    }
                }
            },
        },
        MainCommand::Cleaning { command } => match command {
            CleaningCommands::List => {
                let items = CleaningProcedure::all()
                    .include(CleaningProcedure::fields().taxon_links().taxon())
                    .exec(&mut db)
                    .await?;
                let nitems = items.len();
                let mut tbuilder = TableBuilder::default();
                tbuilder.push_record(["ID", "Name", "Taxa"]);
                for item in items {
                    tbuilder.push_record([
                        item.id.to_string(),
                        item.name,
                        item.taxon_links.get().len().to_string(),
                    ])
                }
                println!("{}", tbuilder.build().with(style::BasicTable));
                println!("\n{nitems} found");
            }
            CleaningCommands::Show { id } => {
                let procedure = CleaningProcedure::filter_by_id(id)
                    .include(CleaningProcedure::fields().taxon_links().taxon())
                    .one()
                    .exec(&mut db)
                    .await?;
                let mut tbuilder = TableBuilder::default();
                tbuilder.push_record(["ID", &procedure.id.to_string()]);
                tbuilder.push_record(["Name", &procedure.name]);
                tbuilder.push_record(["Notes", &procedure.notes.unwrap_or_else(|| "-".into())]);
                tbuilder.push_record([
                    "Taxa",
                    &join_or_default(procedure.taxon_links.get(), "-", |v| {
                        v.taxon.get().reference()
                    }),
                ]);
                tbuilder.push_record(["Instructions", &procedure.instructions]);
                println!("{}", tbuilder.build().with(style::BasicTable));
            }
            CleaningCommands::Add {
                name,
                instructions,
                notes,
            } => {
                let item = CleaningProcedure::create()
                    .name(name)
                    .instructions(instructions)
                    .notes(notes)
                    .exec(&mut db)
                    .await?;
                println!("Added new procedure {}", item.id);
            }
            CleaningCommands::Remove { id, assumeyes } => {
                let item = CleaningProcedure::filter_by_id(id)
                    .include(CleaningProcedure::fields().taxon_links())
                    .one()
                    .exec(&mut db)
                    .await?;
                if assumeyes
                    || inquire::Confirm::new(&format!(
                        "Are you sure you wish to remove cleaning procedure {id}?"
                    ))
                    .with_default(false)
                    .with_help_message(&format!(
                        "It is used by {} taxa",
                        item.taxon_links.get().len()
                    ))
                    .prompt()?
                {
                    CleaningProcedure::delete_by_id(&mut db, id).await?;
                    println!("Removed cleaning procedure {id}");
                }
            }
            CleaningCommands::Modify {
                id,
                name,
                instructions,
                notes,
            } => {
                let mut query = CleaningProcedure::update_by_id(id);
                if let Some(name) = name {
                    query = query.name(name);
                }
                if let Some(instructions) = instructions {
                    query = query.instructions(instructions);
                }
                if let Some(notes) = notes {
                    query = query.notes(notes);
                }
                query.exec(&mut db).await?;
                println!("Modified cleaning procedure {id}");
            }
        },
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
