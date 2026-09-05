use libpropagation::{
    citation::{
        Citation, CleaningProcedureCitation,
        dto::{CitationCompact, CitationDetails},
    },
    cleaning::{CleaningProcedure, dto::CleaningProcedureDetails},
};
use toasty::Db;

use crate::{
    cli::{OutputFormat, citation::CitationCommands},
    util::dialog::confirm,
    views::{
        JsonView, YamlView,
        citation::{CitationDetailsView, CitationListView},
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
    #[command(about = "Manage citations for cleaning procedures")]
    Citations {
        #[arg(help = "A cleaning procedure ID")]
        id: u64,
        #[command(subcommand)]
        command: CitationCommands,
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
                let procedures: Vec<libpropagation::cleaning::dto::CleaningProcedureCompact> =
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
                    confirm(&format!(
                        "Are you sure you wish to remove cleaning procedure {id}?",
                    ))
                    .selected(false)
                    .run()?
                } {
                    CleaningProcedure::delete_by_id(db, id).await?;
                    println!("Removed cleaning procedure {id}");
                }
            }
            TaxonCleaningCommands::Citations { command, id } => {
                command.run_cleaning(db, *id, format).await?
            }
        }
        Ok(())
    }
}

impl CitationCommands {
    pub async fn run_cleaning(
        &self,
        db: &mut Db,
        cleaning_id: u64,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            CitationCommands::Add {
                title,
                url,
                author,
                year,
                access_date,
                container_title,
                doi,
            } => {
                CleaningProcedureCitation::create()
                    .citation(
                        Citation::create()
                            .title(title)
                            .url(url)
                            .author(author)
                            .access_date(access_date)
                            .publication_year(year)
                            .container_title(container_title)
                            .doi(doi)
                            .exec(db)
                            .await?,
                    )
                    .cleaning_id(cleaning_id)
                    .exec(db)
                    .await?;
                load_and_display_cleaning_details(db, format, &cleaning_id).await?;
            }
            CitationCommands::Remove {
                citation_id,
                assumeyes,
            } => {
                if *assumeyes || {
                    load_and_display_citation_details(
                        db,
                        citation_id,
                        cleaning_id,
                        OutputFormat::Text,
                    )
                    .await?;
                    confirm("Do you want to remove this citation?")
                        .selected(false)
                        .run()?
                } {
                    CleaningProcedureCitation::delete_by_citation_id_and_cleaning_id(
                        db,
                        citation_id,
                        cleaning_id,
                    )
                    .await?;
                    Citation::delete_if_unused(db, citation_id).await?;
                    load_and_display_cleaning_details(db, format, &cleaning_id).await?;
                }
            }
            CitationCommands::List => {
                let citations: Vec<CitationCompact> =
                    CleaningProcedureCitation::filter_by_cleaning_id(cleaning_id)
                        .include(CleaningProcedureCitation::fields().citation())
                        .exec(db)
                        .await?
                        .into_iter()
                        .map(|pc| pc.citation.get().into())
                        .collect();
                let output = match format {
                    OutputFormat::Text => CitationListView::new(&citations).render()?,
                    OutputFormat::Json => JsonView::new(&citations).render()?,
                    OutputFormat::Yaml => YamlView::new(&citations).render()?,
                };
                println!("{output}");
            }
            CitationCommands::Show { id } => {
                load_and_display_citation_details(db, id, cleaning_id, format).await?
            }
            CitationCommands::Link { id } => {
                CleaningProcedureCitation::create()
                    .citation_id(id)
                    .cleaning_id(cleaning_id)
                    .exec(db)
                    .await?;
                load_and_display_cleaning_details(db, format, &cleaning_id).await?;
            }
        }
        Ok(())
    }
}

async fn load_and_display_citation_details(
    db: &mut Db,
    citation_id: &u64,
    cleaning_id: u64,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let pc: CitationDetails =
        CleaningProcedureCitation::filter_by_citation_id_and_cleaning_id(citation_id, cleaning_id)
            .include(CleaningProcedureCitation::fields().citation())
            .one()
            .exec(db)
            .await?
            .citation
            .get()
            .into();
    let output = match format {
        OutputFormat::Text => CitationDetailsView::new(&pc, false).render()?,
        OutputFormat::Json => JsonView::new(&pc).render()?,
        OutputFormat::Yaml => YamlView::new(&pc).render()?,
    };
    println!("{output}");
    Ok(())
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
