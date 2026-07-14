use libpropagation::{
    citation::{Citation, CleaningProcedureCitation, dto::CitationDetails},
    collecting::{CleaningProcedure, dto::CleaningProcedureDetails},
};
use toasty::Db;

use crate::{
    cli::OutputFormat,
    util::dialog::confirm,
    views::{
        JsonView, YamlView,
        citation::CitationDetailsView,
        cleaning::{CleaningProcedureDetailsView, CleaningProcedureListView},
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCleaningCommands {
    #[command(about = "Show all seed cleaning procedures for a taxon", alias = "ls")]
    List,
    #[command(about = "Show all seed cleaning procedures for a taxon")]
    Show {
        #[arg(help = "A cleaning procedure ID")]
        procedure_id: u64,
    },
    #[command(about = "Add a new seed cleaning procedure", alias = "new")]
    Add {
        #[arg(short, long, help = "A name for the procedure")]
        name: String,
        #[arg(short, long, help = "A name for the procedure")]
        instructions: String,
        #[arg(long, help = "General notes about the procedure")]
        notes: Option<String>,
    },
    #[command(about = "Modify a seed cleaning procedure", group(clap::ArgGroup::new("cleaning_props").args(["name", "instructions", "notes"]).required(true).multiple(false)), alias="edit")]
    Modify {
        id: u64,
        #[arg(short, long, help = "A name for the procedure")]
        instructions: Option<String>,
        #[arg(short, long, help = "A name for the procedure")]
        name: Option<String>,
        #[arg(long, help = "General notes about the procedure")]
        notes: Option<String>,
    },
    #[command(about = "Remove a seed cleaning procedure")]
    Remove {
        id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    AddCitation {
        #[arg(help = "A cleaning procedure ID")]
        id: u64,
        #[arg(help = "Citation subject")]
        subject: String,
        #[arg(long, help = "A canonical URL for the citation")]
        url: Option<String>,
        #[arg(long, help = "The author being cited")]
        author: Option<String>,
    },
    RemoveCitation {
        #[arg(help = "A cleaning procedure ID")]
        id: u64,
        #[arg(short, long, help = "A citation ID")]
        citation_id: u64,
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
                let procedures: Vec<libpropagation::collecting::dto::CleaningProcedureCompact> =
                    CleaningProcedure::filter_by_taxon_id(taxon_id)
                        .exec(db)
                        .await?
                        .into_iter()
                        .map(Into::into)
                        .collect();

                let output = match format {
                    OutputFormat::Text => CleaningProcedureListView::new(&procedures).render()?,
                    OutputFormat::Json => JsonView::new(&procedures).render()?,
                    OutputFormat::Yaml => YamlView::new(&procedures).render()?,
                };
                println!("{output}");
            }
            TaxonCleaningCommands::Show { procedure_id } => {
                load_and_display_cleaning_details(db, format, procedure_id).await?;
            }
            TaxonCleaningCommands::Add {
                name,
                instructions,
                notes,
            } => {
                let item: CleaningProcedureDetails = CleaningProcedure::create()
                    .taxon_id(taxon_id)
                    .name(name)
                    .instructions(instructions)
                    .notes(notes)
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => CleaningProcedureDetailsView::new(&item).render()?,
                    OutputFormat::Json => JsonView::new(&item).render()?,
                    OutputFormat::Yaml => YamlView::new(&item).render()?,
                };
                println!("{output}");
            }

            TaxonCleaningCommands::Modify {
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
                query.exec(db).await?;
                let item: CleaningProcedureDetails = CleaningProcedure::filter_by_id(id)
                    .include(CleaningProcedure::fields().taxon())
                    .one()
                    .exec(db)
                    .await?
                    .into();
                println!("{}", CleaningProcedureDetailsView::new(&item).render()?);
            }
            TaxonCleaningCommands::Remove { id, assumeyes } => {
                if *assumeyes || {
                    let item: CleaningProcedureDetails = CleaningProcedure::filter_by_id(id)
                        .include(CleaningProcedure::fields().taxon())
                        .one()
                        .exec(db)
                        .await?
                        .into();
                    println!("{}", CleaningProcedureDetailsView::new(&item).render()?);
                    confirm()
                        .with_prompt(format!(
                            "Are you sure you wish to remove cleaning procedure {id}?",
                        ))
                        .default(false)
                        .interact()?
                } {
                    CleaningProcedure::delete_by_id(db, id).await?;
                    println!("Removed cleaning procedure {id}");
                }
            }
            TaxonCleaningCommands::AddCitation {
                id,
                subject,
                url,
                author,
            } => {
                let citation = Citation::create()
                    .text(subject)
                    .url(url)
                    .author(author)
                    .exec(db)
                    .await?;
                CleaningProcedureCitation::create()
                    .citation_id(citation.id)
                    .cleaning_id(id)
                    .exec(db)
                    .await?;
                load_and_display_cleaning_details(db, format, id).await?;
            }
            TaxonCleaningCommands::RemoveCitation {
                id,
                citation_id,
                assumeyes,
            } => {
                if *assumeyes || {
                    let pc: CitationDetails =
                        CleaningProcedureCitation::filter_by_citation_id_and_cleaning_id(
                            citation_id,
                            id,
                        )
                        .include(CleaningProcedureCitation::fields().citation())
                        .one()
                        .exec(db)
                        .await?
                        .citation
                        .get()
                        .into();
                    let output = CitationDetailsView::new(&pc).render()?;
                    println!("{output}");
                    confirm()
                        .with_prompt("Do you want to remove this citation?")
                        .default(false)
                        .interact()?
                } {
                    CleaningProcedureCitation::delete_by_citation_id_and_cleaning_id(
                        db,
                        citation_id,
                        id,
                    )
                    .await?;
                    let citation = Citation::filter_by_id(citation_id)
                        .include(Citation::fields().propagation_procedures())
                        .include(Citation::fields().taxon_propagation_procedures())
                        .include(Citation::fields().cleaning_procedures())
                        .one()
                        .exec(db)
                        .await?;
                    // if the citation is no longer rused, remove it from the database
                    if citation.propagation_procedures.get().is_empty()
                        && citation.taxon_propagation_procedures.get().is_empty()
                        && citation.cleaning_procedures.get().is_empty()
                    {
                        Citation::delete_by_id(db, citation_id).await?;
                    }
                    load_and_display_cleaning_details(db, format, id).await?;
                }
            }
        }
        Ok(())
    }
}

async fn load_and_display_cleaning_details(
    db: &mut Db,
    format: OutputFormat,
    id: &u64,
) -> Result<(), anyhow::Error> {
    let procedure: CleaningProcedureDetails = CleaningProcedure::filter_by_id(id)
        .include(CleaningProcedure::fields().taxon())
        .include(CleaningProcedure::fields().citations())
        .one()
        .exec(db)
        .await?
        .into();
    let output = match format {
        OutputFormat::Text => CleaningProcedureDetailsView::new(&procedure).render()?,
        OutputFormat::Json => JsonView::new(&procedure).render()?,
        OutputFormat::Yaml => YamlView::new(&procedure).render()?,
    };
    println!("{output}");
    Ok(())
}
