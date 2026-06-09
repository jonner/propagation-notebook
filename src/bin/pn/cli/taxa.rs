use propagation_notebook::{collecting::CollectingData, propagation::TaxonProtocol};
use tabled::builder::Builder as TableBuilder;
use toasty::Db;

use crate::style;

pub mod cleaning;

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
        command: TaxonCollectingCommands,
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
        command: TaxonPropagationCommands,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCollectingCommands {
    #[command(about = "Show seed collecting information")]
    Show,
    #[command(about = "Modify seed collecting information for a taxon", group(clap::ArgGroup::new("modify_props").args(["ripening_indicators", "harvesting_notes", "storage_conditions", "storage_life"]).required(true).multiple(true)))]
    Modify {
        #[arg(
            short,
            long,
            help = "What to look for to determine if the seed is ready for collecting"
        )]
        ripening_indicators: Option<String>,
        #[arg(long, help = "Harvesting notes")]
        harvesting_notes: Option<String>,
        #[arg(short, long, help = "Instructions for storing the seed")]
        storage_conditions: Option<String>,
        #[arg(
            short = 'l',
            long,
            help = "How long the seed will stay viable in storage"
        )]
        storage_life: Option<String>,
    },
    #[command(about = "Remove seed collecting information")]
    Remove {
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl TaxonCollectingCommands {
    pub async fn run(&self, db: &mut Db, taxon_id: u64) -> anyhow::Result<()> {
        match self {
            TaxonCollectingCommands::Show => {
                match CollectingData::filter_by_taxon_id(taxon_id)
                    .include(CollectingData::fields().taxon())
                    .one()
                    .exec(db)
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
                            "Harvesting",
                            data.harvesting_notes.as_deref().unwrap_or("-"),
                        ]);
                        tbuilder.push_record([
                            "Storage Conditions",
                            data.storage.as_deref().unwrap_or("-"),
                        ]);
                        tbuilder.push_record([
                            "Storage Life",
                            data.storage_life.as_deref().unwrap_or("-"),
                        ]);
                        println!("{}", tbuilder.build().with(crate::style::DetailTable))
                    }
                    Err(e) if e.is_record_not_found() => println!(
                        "Taxon {taxon_id} does not current have any collecting information defined"
                    ),
                    Err(e) => return Err(e.into()),
                }
            }
            TaxonCollectingCommands::Remove { assumeyes } => {
                if *assumeyes
                    || inquire::Confirm::new(
                        "Are you sure you wish to remove this collecting data?",
                    )
                    .with_default(false)
                    .prompt()?
                {
                    CollectingData::delete_by_taxon_id(db, taxon_id).await?;
                    println!("Removed collecting data {taxon_id}")
                }
            }
            TaxonCollectingCommands::Modify {
                ripening_indicators,
                harvesting_notes,
                storage_conditions,
                storage_life,
            } => {
                // Try to create the object first
                match CollectingData::create()
                    .taxon_id(taxon_id)
                    .ripening_indicators(ripening_indicators)
                    .harvesting_notes(harvesting_notes)
                    .storage(storage_conditions)
                    .storage_life(storage_life)
                    .exec(db)
                    .await
                {
                    Ok(data) => {
                        println!("Added collection information for taxon {}", data.taxon_id)
                    }
                    Err(e) if e.is_condition_failed() => {
                        // the creation likely failed because CollectionData already
                        // exists for taxon_id, which has a unique constraint. Just
                        // update the object
                        let mut query = CollectingData::update_by_taxon_id(taxon_id);
                        if let Some(ripening) = ripening_indicators {
                            query = query.ripening_indicators(ripening);
                        }
                        if let Some(harvesting) = harvesting_notes {
                            query = query.harvesting_notes(harvesting);
                        }
                        if let Some(storage) = storage_conditions {
                            query = query.storage(storage);
                        }
                        if let Some(storage_life) = storage_life {
                            query = query.storage_life(storage_life);
                        }
                        query.exec(db).await?;
                        println!("Modified collection information {taxon_id}");
                    }
                    Err(e) => Err(e)?,
                };
            }
        }
        Ok(())
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum TaxonPropagationCommands {
    #[command(about = "List seed propagation protocols for the taxon")]
    List,
    #[command(about = "Show seed propagation information for the taxon")]
    Show {
        #[arg(
            short,
            long,
            help = "An ID of a propagation protocol ID assigned to this taxon"
        )]
        protocol_id: u64,
    },
    #[command(about = "Assign a new seed propagation protocol to a taxon")]
    Add {
        #[arg(short, long, help = "An ID of a propagation protocol ID")]
        protocol_id: u64,
        #[arg(
            short,
            long,
            help = "Confidence level in this propagation protocol (0-10)",
            value_parser = clap::value_parser!(u8).range(0..=10)
        )]
        confidence: Option<u8>,
        #[arg(
            short,
            long,
            help = "Taxon-specific notes for this propagation protocol"
        )]
        notes: Option<String>,
    },
    #[command(about = "Modify propagation information for a taxon", group(clap::ArgGroup::new("modify_props").args(["confidence", "notes"]).required(true).multiple(false)))]
    Modify {
        #[arg(short, long, help = "A propagation protocol ID assigned to this taxon")]
        protocol_id: u64,
        #[arg(
            short,
            long,
            help = "Confidence level in this propagation protocol (0-10)",
            value_parser = clap::value_parser!(u8).range(0..=10)
        )]
        confidence: Option<u8>,
        #[arg(
            short,
            long,
            help = "Taxon-specific notes for this propagation protocol"
        )]
        notes: Option<String>,
    },
    #[command(about = "Remove propagation information from the taxon")]
    Remove {
        #[arg(short, long, help = "A propagation protocol ID assigned to this taxon")]
        protocol_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl TaxonPropagationCommands {
    pub async fn run(&self, db: &mut Db, taxon_id: u64) -> anyhow::Result<()> {
        match self {
        TaxonPropagationCommands::List => {
            let tps = TaxonProtocol::filter_by_taxon_id(taxon_id)
                .include(TaxonProtocol::fields().taxon())
                .include(TaxonProtocol::fields().protocol())
                .exec(db)
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
        TaxonPropagationCommands::Show { protocol_id } => {
            let tp = TaxonProtocol::filter_by_taxon_id_and_protocol_id(taxon_id, protocol_id)
                .include(TaxonProtocol::fields().taxon())
                .include(TaxonProtocol::fields().protocol())
                .one()
                .exec(db)
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
            tbuilder.push_record(["Taxon-specific notes", tp.notes.as_deref().unwrap_or("-")]);
            tbuilder.push_record(["Protocol", &tp.protocol.get().id.to_string()]);
            println!("{}", tbuilder.build().with(style::DetailTable));
        }
        TaxonPropagationCommands::Add {
            protocol_id,
            confidence,
            notes,
        } => {
            TaxonProtocol::create()
                .protocol_id(protocol_id)
                .taxon_id(taxon_id)
                .confidence(confidence)
                .notes(notes)
                .exec(db)
                .await?;
            println!("Added propagation protocol {protocol_id} to taxon {taxon_id}");
        }
        TaxonPropagationCommands::Modify {
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
            query.exec(db).await?;
            println!("Updated propagation info");
        }
        TaxonPropagationCommands::Remove {
            protocol_id,
            assumeyes,
        } => {
            if *assumeyes
                        || inquire::Confirm::new(
                            "Are you sure you wish to remove this propagation protocol from taxon {taxon_id}?",
                        )
                        .with_default(false)
                        .prompt()?
                    {
                        TaxonProtocol::delete_by_taxon_id_and_protocol_id(
                            db,
                            taxon_id,
                            protocol_id,
                        )
                        .await?;
                        println!("Removed propagation protocol {protocol_id} for taxon {taxon_id}");
                    }
        }
    }
        Ok(())
    }
}
