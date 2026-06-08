use std::collections::HashMap;

use anyhow::anyhow;
use clap::Parser;
use directories::ProjectDirs;
use propagation_notebook::{
    collecting::{CleaningProcedure, CollectingData, TaxonCleaningProcedure},
    propagation::{Protocol, ProtocolType, TaxonProtocol},
    region::{Region, RegionalTaxonStatus},
    taxonomy::{Synonym, Taxon, VernacularName},
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
        taxa::{TaxonCleaningCommands, TaxonCollectingCommands, TaxonCommands},
    },
    import_region::import_region,
};

mod cli;
mod import_region;
mod import_taxa;

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
        MainCommand::Init => {
            match db.push_schema().await {
                Ok(()) => Ok(()),
                Err(e) => if e.is_driver_operation_failed() {
                    // this is the error that occurs when we try to push a schema
                    // after the schema has already been applied
                    Ok(())
                } else {
                    // Report other errors
                    Err(e)
                },
            }?;
        }
        MainCommand::Taxa { command } => match command {
            TaxonCommands::Search { search_string } => {
                tracing::debug!("Searching for exact complete name");
                if let Ok(found) = Taxon::filter(Taxon::fields().complete_name().eq(&search_string))
                    .one()
                    .exec(&mut db)
                    .await
                {
                    println!("found taxon {}", found.reference());
                } else {
                    tracing::debug!("Searching for approximate complete name");
                    let wildcard = format!("%{search_string}%");
                    let taxa = Taxon::filter(Taxon::fields().complete_name().like(&wildcard))
                        .exec(&mut db)
                        .await?;
                    if !taxa.is_empty() {
                        println!("Possible options for '{search_string}':");
                        for t in taxa {
                            println!("- {}", t.reference());
                        }
                    } else {
                        tracing::debug!("Searching for exact scientific synonym");
                        if let Ok(found) =
                            Synonym::filter(Synonym::fields().complete_name().eq(&search_string))
                                .include(Synonym::fields().taxon())
                                .one()
                                .exec(&mut db)
                                .await
                        {
                            println!(
                                "Found '{}' which is a synonym for {}",
                                found.complete_name,
                                found.taxon.get().reference(),
                            );
                        } else {
                            tracing::debug!("Searching for approximate scientific synonyms");
                            let synonyms =
                                Synonym::filter(Synonym::fields().complete_name().like(&wildcard))
                                    .include(Synonym::fields().taxon())
                                    .exec(&mut db)
                                    .await?;
                            if !synonyms.is_empty() {
                                println!("Possible options for '{search_string}':");
                                for syn in synonyms {
                                    println!(
                                        "'{}' is a synonym for {}",
                                        syn.complete_name,
                                        syn.taxon.get().reference(),
                                    );
                                }
                            } else {
                                tracing::debug!("Searching for exact vernacular name");
                                // look up common names
                                if let Ok(vernacular) = VernacularName::filter(
                                    VernacularName::fields().name().eq(&search_string),
                                )
                                .include(VernacularName::fields().taxon())
                                .one()
                                .exec(&mut db)
                                .await
                                {
                                    println!(
                                        "Found {} ({})",
                                        vernacular.taxon.get().reference(),
                                        vernacular.name
                                    );
                                } else {
                                    tracing::debug!("Searching for approximate vernacular names");
                                    let vernaculars = VernacularName::filter(
                                        VernacularName::fields().name().like(&wildcard),
                                    )
                                    .include(VernacularName::fields().taxon())
                                    .exec(&mut db)
                                    .await?;
                                    if !vernaculars.is_empty() {
                                        println!("Possible options for '{search_string}':");
                                        for vernacular in vernaculars {
                                            println!(
                                                "{} ({})",
                                                vernacular.taxon.get().reference(),
                                                vernacular.name,
                                            );
                                        }
                                    } else {
                                        println!("No options found");
                                    }
                                }
                            }
                        }
                    }
                }
            }
            TaxonCommands::Show { id } => {
                let taxon = Taxon::filter_by_id(id)
                    .include(Taxon::fields().parent())
                    .include(Taxon::fields().children())
                    .include(Taxon::fields().vernaculars())
                    .include(Taxon::fields().synonyms())
                    .include(Taxon::fields().regional_statuses().region())
                    .include(Taxon::fields().collecting_data())
                    .include(Taxon::fields().cleaning_procedures().procedure())
                    .include(Taxon::fields().propagation_protocols().protocol())
                    .one()
                    .exec(&mut db)
                    .await?;
                {
                    let mut tbuilder = TableBuilder::default();
                    tbuilder.push_record(["ID", &taxon.id.to_string()]);
                    tbuilder.push_record(["Name", &taxon.complete_name]);
                    tbuilder.push_record(["Rank", &taxon.rank.to_string()]);
                    tbuilder.push_record([
                        "Parent",
                        &taxon
                            .parent
                            .get()
                            .as_ref()
                            .map(|p| format!("{} ({})", p.reference(), p.rank))
                            .unwrap_or_else(|| "-".into()),
                    ]);
                    tbuilder.push_record([
                        "Synonyms",
                        &join_or_default(taxon.synonyms.get(), "-", |v| v.complete_name.clone()),
                    ]);
                    tbuilder.push_record([
                        "Common Name(s)",
                        &join_or_default(taxon.vernaculars.get(), "-", |v| v.name.clone()),
                    ]);
                    tbuilder.push_record([
                        "Child taxa",
                        &join_or_default(taxon.children.get(), "-", |t| {
                            format!("{} ({})", t.reference(), t.rank)
                        }),
                    ]);
                    tbuilder.push_record([
                        "Ripening",
                        taxon
                            .collecting_data
                            .get()
                            .as_ref()
                            .and_then(|d| d.ripening_indicators.as_deref())
                            .unwrap_or("-"),
                    ]);
                    tbuilder.push_record([
                        "Storage Conditions",
                        taxon
                            .collecting_data
                            .get()
                            .as_ref()
                            .and_then(|d| d.storage.as_deref())
                            .unwrap_or("-"),
                    ]);
                    tbuilder.push_record([
                        "Storage Life",
                        taxon
                            .collecting_data
                            .get()
                            .as_ref()
                            .and_then(|d| d.storage_life.as_deref())
                            .unwrap_or("-"),
                    ]);
                    tbuilder.push_record(["Seed Cleaning", &{
                        match taxon.cleaning_procedures.get() {
                            procedures if procedures.is_empty() => "-".to_string(),
                            procedures => {
                                let mut inner_table = TableBuilder::default();
                                inner_table.push_record(["ID", "Name"]);
                                procedures.iter().for_each(|tcp| {
                                    let proc = tcp.procedure.get();
                                    inner_table.push_record([&proc.id.to_string(), &proc.name]);
                                });
                                inner_table.build().with(style::DetailTable).to_string()
                            }
                        }
                    }]);
                    tbuilder.push_record(["Propagation Protocols", &{
                        match taxon.propagation_protocols.get() {
                            tp if tp.is_empty() => "-".to_string(),
                            tps => {
                                let mut inner_table = TableBuilder::default();
                                inner_table.push_record(["ID", "Name", "Type"]);
                                tps.iter().for_each(|tp| {
                                    let protocol = tp.protocol.get();
                                    inner_table.push_record([&protocol.id.to_string(), &protocol.name, &protocol.r#type.to_string()]);
                                });
                                inner_table.build().with(style::BasicTable).to_string()
                            }
                        }
                    }]);
                    tbuilder.push_record([
                        "Regions",
                        &{
                            let regions = taxon.regional_statuses.get();
                            if regions.is_empty() {
                                "-".to_string()
                            } else {
                                let mut inner_table = TableBuilder::default();
                                inner_table.push_record(["ID", "Name", "Origin"]);
                                for rs in regions.iter() {
                                    inner_table.push_record([
                                    rs.region.get().id.to_string(),
                                    rs.region.get().name.clone(),
                                    rs.origin.map(|val| val.to_string()).unwrap_or_else(|| "-".into())
                                    ]);
                                }
                                inner_table.build().with(style::BasicTable).to_string()
                            }},
                    ]);
                    println!("{}", tbuilder.build().with(style::DetailTable));
                    println!();
                }
            }
            TaxonCommands::List { region_id } => match region_id {
                Some(id) => list_regional_taxa(&mut db, id).await?,
                None => {
                    let taxa = Taxon::all()
                        .order_by(Taxon::fields().sequence().asc())
                        .exec(&mut db)
                        .await?;
                    let ntaxa = taxa.len();
                    if taxa.is_empty() {
                        println!("The taxonomy has not been imported. Please download the ITIS taxonomy database from https://www.itis.gov/downloads/index.html and import it with `pn taxa import`")
                    } else {
                        let mut tbuilder = TableBuilder::default();
                        tbuilder.push_record(["ID", "Name"]);
                        for taxon in taxa {
                            tbuilder.push_record([taxon.id.to_string(), taxon.complete_name]);
                        }
                        println!("{}", tbuilder.build().with(style::BasicTable));
                        println!("{} taxa found", ntaxa);
                    }
                }
            },
            TaxonCommands::Cleaning { taxon_id, command } => match command {
                TaxonCleaningCommands::List => {
                    let procedures = TaxonCleaningProcedure::filter_by_taxon_id(taxon_id)
                        .exec(&mut db)
                        .await?;

                    let mut tbuilder = TableBuilder::default();
                    tbuilder.push_record(["Taxon ID", "Procedure ID", "Notes"]);
                    for proc in procedures {
                        tbuilder.push_record([
                            proc.taxon_id.to_string(),
                            proc.procedure_id.to_string(),
                            proc.notes.unwrap_or_else(|| "-".into()),
                        ]);
                    }
                    println!("{}", tbuilder.build().with(style::BasicTable));
                }
                TaxonCleaningCommands::Show { procedure_id } => {
                    let tcp = TaxonCleaningProcedure::filter_by_taxon_id_and_procedure_id(
                        taxon_id,
                        procedure_id,
                    )
                    .include(TaxonCleaningProcedure::fields().taxon())
                    .include(TaxonCleaningProcedure::fields().procedure())
                    .one()
                    .exec(&mut db)
                    .await?;

                    let mut tbuilder = TableBuilder::default();
                    tbuilder.push_record([
                        "Taxon",
                        &format!("{}: {}", tcp.taxon_id, tcp.taxon.get().complete_name),
                    ]);
                    tbuilder.push_record(["Procedure", &tcp.procedure_id.to_string()]);
                    tbuilder.push_record(["Notes", &tcp.notes.unwrap_or_else(|| "-".into())]);
                    println!("{}", tbuilder.build().with(style::DetailTable));
                }
                TaxonCleaningCommands::Add {
                    procedure_id,
                    notes,
                } => {
                    TaxonCleaningProcedure::create()
                        .taxon_id(taxon_id)
                        .procedure_id(procedure_id)
                        .notes(notes)
                        .exec(&mut db)
                        .await?;
                    println!("Procedure {} assigned to taxon {}", taxon_id, procedure_id);
                }
                TaxonCleaningCommands::Modify {
                    procedure_id,
                    notes,
                } => {
                    TaxonCleaningProcedure::update_by_taxon_id_and_procedure_id(
                        taxon_id,
                        procedure_id,
                    )
                    .notes(notes)
                    .exec(&mut db)
                    .await?;
                    println!("Procedure {} updated for taxon {}", procedure_id, taxon_id);
                }
                TaxonCleaningCommands::Remove {
                    procedure_id,
                    assumeyes,
                } => {
                    if assumeyes
                        || inquire::Confirm::new("Are you sure you wish to remove this procedure?")
                            .with_default(false)
                            .prompt()?
                    {
                        TaxonCleaningProcedure::delete_by_taxon_id_and_procedure_id(
                            &mut db,
                            taxon_id,
                            procedure_id,
                        )
                        .await?;
                        println!("Assignment removed");
                    }
                }
            },
            TaxonCommands::Import {
                db_uri,
                authority,
                assumeyes,
            } => {
                let ntaxa = Taxon::all().count().exec(&mut db).await?;
                if assumeyes
                    || inquire::Confirm::new(
                        "Are you sure you wish to import all taxa from the external database?",
                    )
                    .with_default(false)
                    .with_help_message(&format!("The database currently contains {ntaxa} taxa"))
                    .prompt()?
                {
                    // FIXME: we should probably clear the database if the
                    // user confirms rather than re-import a taxonomy into an
                    // existing taxonomy
                    import_taxa::import_taxa(&mut db, &db_uri, authority).await?
                }
            }
            TaxonCommands::Collecting { taxon_id, command } => match command {
                TaxonCollectingCommands::Show => {
                    match CollectingData::filter_by_taxon_id(taxon_id)
                        .include(CollectingData::fields().taxon())
                        .one()
                        .exec(&mut db)
                        .await
                    {
                        Ok(data) => {
                            let mut tbuilder = TableBuilder::default();
                            tbuilder.push_record(["Taxon", &data.taxon.get().reference()]);
                            tbuilder.push_record([
                                "Ripening",
                                data.ripening_indicators.as_deref().unwrap_or("-"),
                            ]);
                            tbuilder.push_record([
                                "Storage Conditions",
                                data.storage.as_deref().unwrap_or("-"),
                            ]);
                            tbuilder.push_record([
                                "Storage Life",
                                data.storage_life.as_deref().unwrap_or("-"),
                            ]);
                            println!("{}", tbuilder.build().with(style::DetailTable))
                        }
                        Err(e) if e.is_record_not_found() => println!(
                            "Taxon {taxon_id} does not current have any collecting information defined"
                        ),
                        Err(e) => return Err(e.into()),
                    }
                }
                TaxonCollectingCommands::Add {
                    ripening_indicators,
                    storage_conditions,
                    storage_life,
                } => {
                    let data = CollectingData::create()
                        .taxon_id(taxon_id)
                        .ripening_indicators(ripening_indicators)
                        .storage(storage_conditions)
                        .storage_life(storage_life)
                        .exec(&mut db)
                        .await?;
                    println!("Added collection information for taxon {}", data.taxon_id);
                }
                TaxonCollectingCommands::Remove { assumeyes } => {
                    if assumeyes
                        || inquire::Confirm::new(
                            "Are you sure you wish to remove this collecting data?",
                        )
                        .with_default(false)
                        .prompt()?
                    {
                        CollectingData::delete_by_taxon_id(&mut db, taxon_id).await?;
                        println!("Removed collecting data {taxon_id}")
                    }
                }
                TaxonCollectingCommands::Modify {
                    ripening_indicators,
                    storage_conditions,
                    storage_life,
                } => {
                    let mut query = CollectingData::update_by_taxon_id(taxon_id);
                    if let Some(ripening) = ripening_indicators {
                        query = query.ripening_indicators(ripening);
                    }
                    if let Some(storage) = storage_conditions {
                        query = query.storage(storage);
                    }
                    if let Some(storage_life) = storage_life {
                        query = query.storage_life(storage_life);
                    }
                    query.exec(&mut db).await?;
                    println!("Modified collection information {taxon_id}");
                }
            },
            TaxonCommands::Propagation { taxon_id, command } => match command {
                cli::taxa::TaxonPropagationCommands::List => {
                    let tps = TaxonProtocol::filter_by_taxon_id(taxon_id)
                        .include(TaxonProtocol::fields().taxon())
                        .include(TaxonProtocol::fields().protocol())
                        .exec(&mut db)
                        .await?;
                    let mut tbuilder = TableBuilder::default();
                    tbuilder.push_record(["Taxon", "Protocol", "Confidence", "Notes"]);
                    for tp in tps {
                        tbuilder.push_record([
                            &tp.taxon.get().reference(),
                            &tp.protocol.get().id.to_string(),
                            tp.confidence
                                .map(|v| v.to_string())
                                .as_deref()
                                .unwrap_or("-"),
                            tp.notes.as_deref().unwrap_or("-"),
                        ])
                    }
                    println!("{}", tbuilder.build().with(style::BasicTable));
                }
                cli::taxa::TaxonPropagationCommands::Show { protocol_id } => {
                    let tp =
                        TaxonProtocol::filter_by_taxon_id_and_protocol_id(taxon_id, protocol_id)
                            .include(TaxonProtocol::fields().taxon())
                            .include(TaxonProtocol::fields().protocol())
                            .one()
                            .exec(&mut db)
                            .await?;
                    let mut tbuilder = TableBuilder::default();
                    tbuilder.push_record(["Taxon", &tp.taxon.get().reference()]);
                    tbuilder.push_record([
                        "Confidence",
                        tp.confidence
                            .map(|v| v.to_string())
                            .as_deref()
                            .unwrap_or("-"),
                    ]);
                    tbuilder
                        .push_record(["Taxon-specific notes", tp.notes.as_deref().unwrap_or("-")]);
                    tbuilder.push_record(["Protocol", &tp.protocol.get().id.to_string()]);
                    println!("{}", tbuilder.build().with(style::DetailTable));
                }
                cli::taxa::TaxonPropagationCommands::Add {
                    protocol_id,
                    confidence,
                    notes,
                } => {
                    TaxonProtocol::create()
                        .protocol_id(protocol_id)
                        .taxon_id(taxon_id)
                        .confidence(confidence)
                        .notes(notes)
                        .exec(&mut db)
                        .await?;
                    println!("Added propagation protocol {protocol_id} to taxon {taxon_id}");
                }
                cli::taxa::TaxonPropagationCommands::Modify {
                    protocol_id,
                    confidence,
                    notes,
                } => {
                    let mut query =
                        TaxonProtocol::update_by_taxon_id_and_protocol_id(taxon_id, protocol_id);
                    if let Some(confidence) = confidence {
                        query = query.confidence(confidence);
                    } else if let Some(notes) = notes {
                        query = query.notes(notes);
                    }
                    query.exec(&mut db).await?;
                    println!("Updated propagation info");
                }
                cli::taxa::TaxonPropagationCommands::Remove {
                    protocol_id,
                    assumeyes,
                } => {
                    if assumeyes
                        || inquire::Confirm::new(
                            "Are you sure you wish to remove this propagation protocol from taxon {taxon_id}?",
                        )
                        .with_default(false)
                        .prompt()?
                    {
                        TaxonProtocol::delete_by_taxon_id_and_protocol_id(
                            &mut db,
                            taxon_id,
                            protocol_id,
                        )
                        .await?;
                        println!("Removed propagation protocol {protocol_id} for taxon {taxon_id}");
                    }
                }
            },
        },
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
