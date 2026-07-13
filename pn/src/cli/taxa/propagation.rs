use libpropagation::taxonomy::{
    TaxonPropagationProcedure,
    dto::{TaxonPropagationProcedureCompact, TaxonPropagationProcedureDetails},
};

use toasty::Db;

use crate::{
    cli::OutputFormat,
    views::{
        JsonView, YamlView,
        propagation::{
            TaxonPropagationProcedureListView, TaxonPropagationPropagationProcedureDetailView,
        },
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum TaxonPropagationCommands {
    #[command(about = "List seed propagation procedure for the taxon")]
    List,
    #[command(about = "Show seed propagation information for the taxon")]
    Show {
        #[arg(help = "An ID of a propagation procedure ID assigned to this taxon")]
        propagation_id: u64,
    },
    #[command(about = "Assign a new seed propagation procedure to a taxon")]
    Add {
        #[arg(help = "An ID of a propagation procedure ID")]
        propagation_id: u64,
        #[arg(
            long,
            help = "Confidence level in this propagation procedure (0-10)",
            value_parser = clap::value_parser!(u8).range(0..=10)
        )]
        confidence: Option<u8>,
        #[arg(
            short,
            long,
            help = "Taxon-specific notes for this propagation procedure"
        )]
        notes: Option<String>,
    },
    #[command(about = "Modify propagation information for a taxon", group(clap::ArgGroup::new("modify_props").args(["confidence", "notes"]).required(true).multiple(true)))]
    Modify {
        #[arg(help = "A propagation procedure ID assigned to this taxon")]
        propagation_id: u64,
        #[arg(
            long,
            help = "Confidence level in this propagation procedure (0-10)",
            value_parser = clap::value_parser!(u8).range(0..=10)
        )]
        confidence: Option<u8>,
        #[arg(
            short,
            long,
            help = "Taxon-specific notes for this propagation procedure"
        )]
        notes: Option<String>,
    },
    #[command(about = "Remove propagation information from the taxon")]
    Remove {
        #[arg(help = "A propagation procedure ID assigned to this taxon")]
        propagation_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl TaxonPropagationCommands {
    pub async fn run(
        &self,
        db: &mut Db,
        taxon_id: u64,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
        TaxonPropagationCommands::List => {
            let tps: Vec<TaxonPropagationProcedureCompact> = TaxonPropagationProcedure::filter_by_taxon_id(taxon_id)
                .include(TaxonPropagationProcedure::fields().taxon())
                .include(TaxonPropagationProcedure::fields().propagation())
                .exec(db)
                .await?.into_iter().map(Into::into).collect();
            let output = match format {
                OutputFormat::Text => TaxonPropagationProcedureListView::new(&tps).render()?,
                OutputFormat::Json => JsonView::new(&tps).render()?,
                OutputFormat::Yaml => YamlView::new(&tps).render()?,
            };
            println!("{output}");
        }
        TaxonPropagationCommands::Show { propagation_id } => {
            load_and_show_taxon_propagation_details(db, taxon_id, format, propagation_id).await?;
        }
        TaxonPropagationCommands::Add {
            propagation_id,
            confidence,
            notes,
        } => {
            let tp: TaxonPropagationProcedureDetails = TaxonPropagationProcedure::create()
                .propagation_id(propagation_id)
                .taxon_id(taxon_id)
                .confidence(confidence)
                .notes(notes)
                .exec(db)
                .await?.into();
            let output = match format {
                OutputFormat::Text => TaxonPropagationPropagationProcedureDetailView::new(&tp).render()?,
                OutputFormat::Json => JsonView::new(&tp).render()?,
                OutputFormat::Yaml => YamlView::new(&tp).render()?,
            };
            println!("{output}");
        }
        TaxonPropagationCommands::Modify {
            propagation_id,
            confidence,
            notes,
        } => {
            let mut query =
                TaxonPropagationProcedure::update_by_taxon_id_and_propagation_id(taxon_id, propagation_id);
            if let Some(confidence) = confidence {
                query = query.confidence(confidence);
            }
            if let Some(notes) = notes {
                query = query.notes(notes);
            }
            query.exec(db).await?;
            load_and_show_taxon_propagation_details(db, taxon_id, format, propagation_id).await?;
        }
        TaxonPropagationCommands::Remove {
            propagation_id,
            assumeyes,
        } => {
            if *assumeyes
                        || dialoguer::Confirm::new().with_prompt(
                            &format!("Are you sure you wish to remove this propagation procedure from taxon {taxon_id}?"),
                        )
                        .default(false)
                        .interact()?
                    {
                        TaxonPropagationProcedure::delete_by_taxon_id_and_propagation_id(
                            db,
                            taxon_id,
                            propagation_id,
                        )
                        .await?;
                        println!("Removed propagation procedure {propagation_id} for taxon {taxon_id}");
                    }
        }
    }
        Ok(())
    }
}

async fn load_and_show_taxon_propagation_details(
    db: &mut Db,
    taxon_id: u64,
    format: OutputFormat,
    propagation_id: &u64,
) -> Result<(), anyhow::Error> {
    let tp: TaxonPropagationProcedureDetails =
        TaxonPropagationProcedure::filter_by_taxon_id_and_propagation_id(taxon_id, propagation_id)
            .include(TaxonPropagationProcedure::fields().taxon())
            .include(TaxonPropagationProcedure::fields().propagation())
            .one()
            .exec(db)
            .await?
            .into();
    let output = match format {
        OutputFormat::Text => TaxonPropagationPropagationProcedureDetailView::new(&tp).render()?,
        OutputFormat::Json => JsonView::new(&tp).render()?,
        OutputFormat::Yaml => YamlView::new(&tp).render()?,
    };
    println!("{output}");
    Ok(())
}
