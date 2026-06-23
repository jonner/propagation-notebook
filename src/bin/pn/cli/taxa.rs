use propagation_notebook::{
    collecting::TaxonCleaningProcedure,
    propagation::TaxonProtocol,
    region::RegionalTaxonStatus,
    taxonomy::{Synonym, Taxon, TaxonNote, TaxonomicAuthority, VernacularName},
};
use toasty::Db;

use crate::{
    cli::print_regional_taxa_table,
    style,
    util::{IndicatifImportProgress, join_or_default},
};

pub mod cleaning;
pub mod collecting;
pub mod event;
pub mod note;
pub mod propagation;

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCommands {
    #[command(about = "Print a list of all taxa")]
    List {
        #[arg(short, long, help = "Show only taxa in the specified region")]
        region_id: Option<u64>,
        #[arg(long, help = "Show only taxa with custom data", hide = true)]
        has_data: bool,
    },
    #[command(about = "Show detailed information about a Taxon")]
    Show { id: u64 },
    #[command(about = "Search for a taxon")]
    Search { search_string: String },
    #[command(about = "Import a new taxonomy for use with this tool")]
    Import {
        #[arg(help = "A URI to the external taxonomy database")]
        db_uri: String,
        #[arg(
            short,
            long,
            help = "The creator of the database",
            value_enum,
            default_value_t = TaxonomicAuthority::Itis
        )]
        authority: TaxonomicAuthority,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    #[command(about = "Manage collecting information for a taxon")]
    Collecting {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: collecting::TaxonCollectingCommands,
    },
    #[command(about = "Manage cleaning information for a taxon")]
    Cleaning {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: cleaning::TaxonCleaningCommands,
    },
    #[command(about = "Manage cleaning information for a taxon")]
    Propagation {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: propagation::TaxonPropagationCommands,
    },
    #[command(about = "Manage notes for a taxon")]
    Notes {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: note::TaxonNoteCommands,
    },
    #[command(about = "Harvest events for a taxon")]
    HarvestEvents {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: event::TaxonHarvestEventCommands,
    },
}

impl TaxonCommands {
    pub async fn run(&self, db: &mut Db) -> anyhow::Result<()> {
        match self {
            TaxonCommands::Search { search_string } => {
                let wildcard = format!("%{search_string}%");
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Name", "Common Names", "Synonym"]);

                if let Ok(found) = Taxon::filter(
                    Taxon::fields()
                        .complete_name()
                        .like(&wildcard)
                        .or(Taxon::fields()
                            .vernaculars()
                            .any(VernacularName::fields().name().like(&wildcard)))
                        .or(Taxon::fields()
                            .synonyms()
                            .any(Synonym::fields().complete_name().like(&wildcard))),
                )
                .order_by(Taxon::fields().sequence().asc())
                .include(Taxon::fields().vernaculars())
                .include(Taxon::fields().synonyms())
                .exec(db)
                .await
                {
                    for t in found {
                        tbuilder.push_record([
                            t.id.to_string(),
                            t.complete_name,
                            t.vernaculars
                                .get()
                                .iter()
                                .map(|v| v.name.as_str())
                                .collect::<Vec<_>>()
                                .join("\n"),
                            t.synonyms
                                .get()
                                .iter()
                                .filter_map(|s| {
                                    match s
                                        .complete_name
                                        .to_lowercase()
                                        .contains(&search_string.to_lowercase())
                                    {
                                        true => Some(s.complete_name.as_str()),
                                        false => None,
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                        ]);
                    }
                }

                println!("{}", tbuilder.build().with(style::ListTable));
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
                    .include(Taxon::fields().harvest_events().location())
                    .include(Taxon::fields().notes())
                    .one()
                    .exec(db)
                    .await?;
                {
                    let mut tbuilder = tabled::builder::Builder::default();
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
                        "Harvesting Notes",
                        taxon
                            .collecting_data
                            .get()
                            .as_ref()
                            .and_then(|d| d.harvesting_notes.as_deref())
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
                                let mut inner_table = tabled::builder::Builder::default();
                                inner_table.push_record(["ID", "Name"]);
                                procedures.iter().for_each(|tcp| {
                                    let proc = tcp.procedure.get();
                                    inner_table.push_record([&proc.id.to_string(), &proc.name]);
                                });
                                inner_table.build().with(style::DetailTable).to_string() + "\n"
                            }
                        }
                    }]);
                    tbuilder.push_record(["Propagation Protocols", &{
                        match taxon.propagation_protocols.get() {
                            tp if tp.is_empty() => "-".to_string(),
                            tps => {
                                let mut inner_table = tabled::builder::Builder::default();
                                inner_table.push_record(["ID", "Name", "Type"]);
                                tps.iter().for_each(|tp| {
                                    let protocol = tp.protocol.get();
                                    inner_table.push_record([
                                        &protocol.id.to_string(),
                                        &protocol.name,
                                        &protocol.r#type.to_string(),
                                    ]);
                                });
                                inner_table.build().with(style::ListTable).to_string() + "\n"
                            }
                        }
                    }]);
                    tbuilder.push_record(["Regions", &{
                        let regions = taxon.regional_statuses.get();
                        if regions.is_empty() {
                            "-".to_string()
                        } else {
                            let mut inner_table = tabled::builder::Builder::default();
                            inner_table.push_record(["ID", "Name", "Origin", "Harvest Window"]);
                            for rs in regions.iter() {
                                inner_table.push_record([
                                    rs.region.get().id.to_string(),
                                    rs.region.get().name.clone(),
                                    rs.origin
                                        .map(|val| val.to_string())
                                        .unwrap_or_else(|| "-".into()),
                                    rs.harvest_window.to_string(),
                                ]);
                            }
                            inner_table.build().with(style::ListTable).to_string() + "\n"
                        }
                    }]);
                    tbuilder.push_record(["Harvesting Events", &{
                        let events = taxon.harvest_events.get();
                        if events.is_empty() {
                            "-".to_string()
                        } else {
                            let mut inner_table = tabled::builder::Builder::default();
                            inner_table.push_record(["ID", "Date", "Location"]);
                            for event in events.iter() {
                                inner_table.push_record([
                                    &event.id.to_string(),
                                    &event.date.to_string(),
                                    &event.location.get().reference(),
                                ]);
                            }
                            inner_table.build().with(style::ListTable).to_string() + "\n"
                        }
                    }]);
                    tbuilder.push_record(["Notes", &{
                        let notes = taxon.notes.get();
                        if notes.is_empty() {
                            "-".to_string()
                        } else {
                            let mut inner_table = tabled::builder::Builder::default();
                            inner_table.push_record(["ID", "Text"]);
                            for note in notes.iter() {
                                inner_table.push_record([&note.id.to_string(), &note.text]);
                            }
                            inner_table.build().with(style::ListTable).to_string() + "\n"
                        }
                    }]);
                    println!("{}", tbuilder.build().with(style::DetailTable));
                    println!();
                }
            }
            TaxonCommands::List {
                region_id,
                has_data,
            } => match region_id {
                Some(id) => {
                    let region_id = *id;
                    let regional_statuses = RegionalTaxonStatus::filter(
                        RegionalTaxonStatus::fields().region_id().eq(region_id),
                    )
                    // FIXME: We want to order by a taxon sequence, but
                    // toasty doesn't yet support ordering by data in a relation
                    .exec(db)
                    .await?;
                    print_regional_taxa_table(db, regional_statuses).await?;
                }
                None => {
                    let taxa = if *has_data {
                        Taxon::filter(
                            Taxon::fields()
                                .collecting_data()
                                .id()
                                .gt(0)
                                .or(Taxon::fields().regional_statuses().any(
                                    RegionalTaxonStatus::fields()
                                        .harvest_window()
                                        .start_doy()
                                        .is_some()
                                        .or(RegionalTaxonStatus::fields()
                                            .harvest_window()
                                            .end_doy()
                                            .is_some()),
                                ))
                                .or(Taxon::fields()
                                    .cleaning_procedures()
                                    .any(TaxonCleaningProcedure::fields().taxon_id().gt(0)))
                                .or(Taxon::fields()
                                    .propagation_protocols()
                                    .any(TaxonProtocol::fields().taxon_id().gt(0)))
                                .or(Taxon::fields()
                                    .notes()
                                    .any(TaxonNote::fields().taxon_id().gt(0)))
                                .or(Taxon::fields()
                                    .harvest_events()
                                    .any(TaxonNote::fields().taxon_id().gt(0))),
                        )
                        .order_by(Taxon::fields().sequence().asc())
                        .exec(db)
                        .await?
                    } else {
                        let taxa = Taxon::all()
                            .order_by(Taxon::fields().sequence().asc())
                            .exec(db)
                            .await?;
                        if taxa.is_empty() {
                            println!(
                                "The taxonomy has not been imported. Please download the ITIS taxonomy database from https://www.itis.gov/downloads/index.html and import it with `pn taxa import`"
                            )
                        }
                        taxa
                    };
                    let ntaxa = taxa.len();
                    let mut tbuilder = tabled::builder::Builder::default();
                    tbuilder.push_record(["ID", "Name"]);
                    for taxon in taxa {
                        tbuilder.push_record([taxon.id.to_string(), taxon.complete_name]);
                    }
                    println!("{}", tbuilder.build().with(style::ListTable));
                    println!("{} taxa found", ntaxa);
                }
            },
            TaxonCommands::Import {
                db_uri,
                authority,
                assumeyes,
            } => {
                let ntaxa = Taxon::all().count().exec(db).await?;
                if *assumeyes
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
                    propagation_notebook::taxonomy::import(
                        db,
                        db_uri,
                        *authority,
                        &mut IndicatifImportProgress::default(),
                    )
                    .await?
                }
            }
            TaxonCommands::Cleaning { taxon_id, command } => command.run(db, *taxon_id).await?,
            TaxonCommands::Collecting { taxon_id, command } => command.run(db, *taxon_id).await?,
            TaxonCommands::Propagation { taxon_id, command } => command.run(db, *taxon_id).await?,
            TaxonCommands::Notes { taxon_id, command } => command.run(db, *taxon_id).await?,
            TaxonCommands::HarvestEvents { taxon_id, command } => {
                command.run(db, *taxon_id).await?
            }
        }
        Ok(())
    }
}
