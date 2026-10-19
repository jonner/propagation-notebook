use std::path::PathBuf;

use libpropagation::{
    cleaning::CleaningProcedure,
    region::{Region, dto::FullRegion},
    taxonomy::TaxonomicAuthority,
};
use toasty::Db;

use crate::{
    cli::OutputFormat,
    util::IndicatifImportProgress,
    views::{JsonView, YamlView, cleaning::CleaningProcedureListView, regions::RegionDetailsView},
};

#[derive(Debug, clap::Subcommand)]
pub enum ImportCommands {
    #[command(about = "Import a new region to the database")]
    Region {
        #[arg(help = "A path to a yaml file describing a region")]
        path: PathBuf,
    },
    #[command(about = "Import a new taxonomy for use with this tool")]
    Taxonomy {
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
    },
    #[command(about = "Import cleaning procedures")]
    CleaningProcedures {
        #[arg(help = "A path to a yaml file of cleaning definitions")]
        path: PathBuf,
    },
}

impl ImportCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            ImportCommands::Region { path } => {
                let file_reader = std::fs::OpenOptions::new().read(true).open(path)?;
                let region: FullRegion =
                    Region::import(db, file_reader, &mut IndicatifImportProgress::default())
                        .await?
                        .into();
                let output = match format {
                    OutputFormat::Text => RegionDetailsView::new(&region).render()?,
                    OutputFormat::Json => JsonView::new(&region).render()?,
                    OutputFormat::Yaml => YamlView::new(&region).render()?,
                };
                println!("{output}");
            }
            ImportCommands::Taxonomy { db_uri, authority } => {
                libpropagation::taxonomy::import(
                    db,
                    db_uri,
                    *authority,
                    &mut IndicatifImportProgress::default(),
                )
                .await?
            }
            ImportCommands::CleaningProcedures { path } => {
                let file_reader = std::fs::OpenOptions::new().read(true).open(path)?;
                let procedures: Vec<libpropagation::cleaning::dto::CleaningProcedureCompact> =
                    CleaningProcedure::import(
                        db,
                        file_reader,
                        &mut IndicatifImportProgress::default(),
                    )
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect();

                let output = match format {
                    OutputFormat::Text => CleaningProcedureListView::new(&procedures).render()?,
                    OutputFormat::Json => JsonView::new(&procedures).render()?,
                    OutputFormat::Yaml => YamlView::new(&procedures).render()?,
                };
                let n = procedures.len();
                println!("Imported {n} procedures:\n{output}");
            }
        }
        Ok(())
    }
}
