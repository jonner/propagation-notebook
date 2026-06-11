use propagation_notebook::propagation::TaxonProtocol;

use tabled::builder::Builder as TableBuilder;
use toasty::Db;

use crate::style;

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
                    &tp.protocol.get().reference(),
                    tp.confidence
                        .map(|v| v.to_string())
                        .as_deref()
                        .unwrap_or("-"),
                    tp.notes.as_deref().unwrap_or("-"),
                ])
            }
            println!("{}", tbuilder.build().with(style::ListTable));
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
