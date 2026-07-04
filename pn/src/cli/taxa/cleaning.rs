use libpropagation::{
    collecting::TaxonCleaningProcedure, taxonomy::dto::TaxonCleaningProcedureDetails,
};

use toasty::Db;

use crate::{
    cli::OutputFormat,
    views::{
        JsonView, YamlView,
        cleaning::{TaxonCleaningProcedureDetailView, TaxonCleaningProcedureListView},
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCleaningCommands {
    #[command(about = "Show all seed cleaning procedures for a taxon")]
    List,
    #[command(about = "Show all seed cleaning procedures for a taxon")]
    Show {
        #[arg(help = "A cleaning procedure ID")]
        procedure_id: u64,
    },
    #[command(about = "Associate a taxon with a seed cleaning procedure")]
    Add {
        #[arg(help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(short, long, help = "Taxon-specific notes for this procedure")]
        notes: Option<String>,
    },
    #[command(about = "Modify taxon-specific information seed cleaning information", group(clap::ArgGroup::new("modify_props").args(["notes"]).required(true).multiple(true)))]
    Modify {
        #[arg(help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(short, long, help = "Taxon-specific notes for this procedure")]
        notes: Option<String>,
    },
    #[command(about = "Remove a cleaning procedure from the specified taxon")]
    Remove {
        #[arg(help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl TaxonCleaningCommands {
    pub async fn run(
        &self,
        db: &mut Db,
        taxon_id: u64,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            TaxonCleaningCommands::List => {
                let procedures: Vec<libpropagation::taxonomy::dto::TaxonCleaningProcedureCompact> =
                    TaxonCleaningProcedure::filter_by_taxon_id(taxon_id)
                        .exec(db)
                        .await?
                        .into_iter()
                        .map(Into::into)
                        .collect();

                let output = match format {
                    OutputFormat::Text => {
                        TaxonCleaningProcedureListView::new(&procedures).render()?
                    }
                    OutputFormat::Json => JsonView::new(&procedures).render()?,
                    OutputFormat::Yaml => YamlView::new(&procedures).render()?,
                };
                println!("{output}");
            }
            TaxonCleaningCommands::Show { procedure_id } => {
                load_and_show_taxon_cleaning_details(db, taxon_id, format, procedure_id).await?;
            }
            TaxonCleaningCommands::Add {
                procedure_id,
                notes,
            } => {
                let tcp: TaxonCleaningProcedureDetails = TaxonCleaningProcedure::create()
                    .taxon_id(taxon_id)
                    .procedure_id(procedure_id)
                    .notes(notes)
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => TaxonCleaningProcedureDetailView::new(&tcp).render()?,
                    OutputFormat::Json => JsonView::new(&tcp).render()?,
                    OutputFormat::Yaml => YamlView::new(&tcp).render()?,
                };
                println!("{output}");
            }
            TaxonCleaningCommands::Modify {
                procedure_id,
                notes,
            } => {
                let mut query = TaxonCleaningProcedure::update_by_taxon_id_and_procedure_id(
                    taxon_id,
                    procedure_id,
                );
                if let Some(notes) = notes {
                    query = query.notes(notes)
                }
                query.exec(db).await?;
                load_and_show_taxon_cleaning_details(db, taxon_id, format, procedure_id).await?;
            }
            TaxonCleaningCommands::Remove {
                procedure_id,
                assumeyes,
            } => {
                if *assumeyes
                    || inquire::Confirm::new("Are you sure you wish to remove this procedure?")
                        .with_default(false)
                        .prompt()?
                {
                    TaxonCleaningProcedure::delete_by_taxon_id_and_procedure_id(
                        db,
                        taxon_id,
                        procedure_id,
                    )
                    .await?;
                    println!("Assignment removed");
                }
            }
        }
        Ok(())
    }
}

async fn load_and_show_taxon_cleaning_details(
    db: &mut Db,
    taxon_id: u64,
    format: OutputFormat,
    procedure_id: &u64,
) -> Result<(), anyhow::Error> {
    let tcp: TaxonCleaningProcedureDetails =
        TaxonCleaningProcedure::filter_by_taxon_id_and_procedure_id(taxon_id, procedure_id)
            .include(TaxonCleaningProcedure::fields().taxon())
            .include(TaxonCleaningProcedure::fields().procedure())
            .one()
            .exec(db)
            .await?
            .into();
    let output = match format {
        OutputFormat::Text => TaxonCleaningProcedureDetailView::new(&tcp).render()?,
        OutputFormat::Json => JsonView::new(&tcp).render()?,
        OutputFormat::Yaml => YamlView::new(&tcp).render()?,
    };
    println!("{output}");
    Ok(())
}
