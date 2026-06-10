use propagation_notebook::{
    collecting::TaxonCleaningProcedure,
    propagation::TaxonProtocol,
    region::RegionalTaxonStatus,
    taxonomy::{Synonym, Taxon, VernacularName},
};
use toasty::Db;

use crate::{cli::list_regional_taxa, style, util::join_or_default};

pub mod cleaning;
pub mod collecting;
mod import;
pub mod propagation;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum TaxonomicAuthority {
    Itis,
}

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
}

impl TaxonCommands {
    pub async fn run(&self, db: &mut Db) -> anyhow::Result<()> {
        match self {
            TaxonCommands::Search { search_string } => {
                tracing::debug!("Searching for exact complete name");
                if let Ok(found) = Taxon::filter(Taxon::fields().complete_name().eq(search_string))
                    .one()
                    .exec(db)
                    .await
                {
                    println!("found taxon {}", found.reference());
                } else {
                    tracing::debug!("Searching for approximate complete name");
                    let wildcard = format!("%{search_string}%");
                    let taxa = Taxon::filter(Taxon::fields().complete_name().like(&wildcard))
                        .exec(db)
                        .await?;
                    if !taxa.is_empty() {
                        println!("Possible options for '{search_string}':");
                        for t in taxa {
                            println!("- {}", t.reference());
                        }
                    } else {
                        tracing::debug!("Searching for exact scientific synonym");
                        if let Ok(found) =
                            Synonym::filter(Synonym::fields().complete_name().eq(search_string))
                                .include(Synonym::fields().taxon())
                                .one()
                                .exec(db)
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
                                    .exec(db)
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
                                    VernacularName::fields().name().eq(search_string),
                                )
                                .include(VernacularName::fields().taxon())
                                .one()
                                .exec(db)
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
                                    .exec(db)
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
                        "Harvesting",
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
                                inner_table.build().with(style::DetailTable).to_string()
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
                                inner_table.build().with(style::BasicTable).to_string()
                            }
                        }
                    }]);
                    tbuilder.push_record(["Regions", &{
                        let regions = taxon.regional_statuses.get();
                        if regions.is_empty() {
                            "-".to_string()
                        } else {
                            let mut inner_table = tabled::builder::Builder::default();
                            inner_table.push_record(["ID", "Name", "Origin"]);
                            for rs in regions.iter() {
                                inner_table.push_record([
                                    rs.region.get().id.to_string(),
                                    rs.region.get().name.clone(),
                                    rs.origin
                                        .map(|val| val.to_string())
                                        .unwrap_or_else(|| "-".into()),
                                ]);
                            }
                            inner_table.build().with(style::BasicTable).to_string()
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
                Some(id) => list_regional_taxa(db, *id).await?,
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
                                        .start()
                                        .is_some()
                                        .or(RegionalTaxonStatus::fields()
                                            .harvest_window()
                                            .end()
                                            .is_some()),
                                ))
                                .or(Taxon::fields()
                                    .cleaning_procedures()
                                    .any(TaxonCleaningProcedure::fields().taxon_id().gt(0)))
                                .or(Taxon::fields()
                                    .propagation_protocols()
                                    .any(TaxonProtocol::fields().taxon_id().gt(0))),
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
                    println!("{}", tbuilder.build().with(style::BasicTable));
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
                    import::import_taxa(db, db_uri, *authority).await?
                }
            }
            TaxonCommands::Cleaning { taxon_id, command } => command.run(db, *taxon_id).await?,
            TaxonCommands::Collecting { taxon_id, command } => command.run(db, *taxon_id).await?,
            TaxonCommands::Propagation { taxon_id, command } => command.run(db, *taxon_id).await?,
        }
        Ok(())
    }
}
