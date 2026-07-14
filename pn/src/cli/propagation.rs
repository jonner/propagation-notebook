use std::path::PathBuf;

use libpropagation::{
    citation::{Citation, PropagationProcedureCitation, dto::CitationDetails},
    propagation::{
        ProcedureType, PropagationProcedure,
        dto::{PropagationProcedureCompact, PropagationProcedureDetails},
    },
};
use serde::Deserialize;
use toasty::Db;

use crate::{
    cli::OutputFormat,
    util::dialog::confirm,
    views::{
        JsonView, YamlView,
        citation::CitationDetailsView,
        propagation::{PropagationProcedureDetailView, PropagationProcedureListView},
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum PropagationCommands {
    #[command(about = "List all seed propagation procedures", alias = "ls")]
    List {
        #[arg(
            short,
            long,
            value_enum,
            help = "limit list to the selected procedure type"
        )]
        r#type: Option<ProcedureType>,
    },
    #[command(about = "Show a seed propagation procedure")]
    Show { id: u64 },
    #[command(about = "Add a seed propagation procedure", alias = "new")]
    Add {
        #[arg(help = "A short name for the procedure")]
        name: String,
        #[arg(short, long, value_enum)]
        r#type: ProcedureType,
        #[arg(long, help = "Instructions for this procedure")]
        instructions: String,
        #[arg(long, help = "Additional notes for this procedure")]
        notes: Option<String>,
    },
    #[command(about = "Modify a seed propagation procedure", group(clap::ArgGroup::new("modify_fields").args(["name", "type", "notes", "instructions"]).required(true).multiple(true)), alias="edit")]
    Modify {
        #[arg(help = "A procedure ID")]
        id: u64,
        #[arg(short, long, help = "A short name for the procedure")]
        name: Option<String>,
        #[arg(short, long, value_enum)]
        r#type: Option<ProcedureType>,
        #[arg(long, help = "Instructions for this procedure")]
        instructions: Option<String>,
        #[arg(long, help = "Additional notes for this procedure")]
        notes: Option<String>,
    },
    #[command(about = "Remove a seed propagation procedure")]
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
        #[arg(help = "A procedure ID")]
        propagation_id: u64,
        #[arg(help = "Citation subject")]
        subject: String,
        #[arg(long, help = "A canonical URL for the citation")]
        url: Option<String>,
        #[arg(long, help = "The author being cited")]
        author: Option<String>,
    },
    RemoveCitation {
        #[arg(help = "A procedure ID")]
        propagation_id: u64,
        #[arg(short, long, help = "A citation ID")]
        citation_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    #[command(about = "Import seed propagation procedures from YAML")]
    Import { path: PathBuf },
}

impl PropagationCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            PropagationCommands::List { r#type } => {
                let mut query = PropagationProcedure::all();
                if let Some(t) = r#type {
                    query = query.filter(PropagationProcedure::fields().r#type().eq(t));
                }
                let procedures: Vec<PropagationProcedureCompact> =
                    query.exec(db).await?.into_iter().map(Into::into).collect();
                let output = match format {
                    OutputFormat::Text => {
                        PropagationProcedureListView::new(&procedures).render()?
                    }
                    OutputFormat::Json => JsonView::new(&procedures).render()?,
                    OutputFormat::Yaml => YamlView::new(&procedures).render()?,
                };
                println!("{output}");
            }
            PropagationCommands::Show { id } => {
                load_and_display_propagation_details(db, format, id).await?;
            }
            PropagationCommands::Add {
                name,
                r#type,
                instructions,
                notes,
            } => {
                let item: PropagationProcedureDetails = PropagationProcedure::create()
                    .name(name)
                    .r#type(r#type)
                    .instructions(instructions)
                    .notes(notes)
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => PropagationProcedureDetailView::new(&item).render()?,
                    OutputFormat::Json => JsonView::new(&item).render()?,
                    OutputFormat::Yaml => YamlView::new(&item).render()?,
                };
                println!("{output}");
            }
            PropagationCommands::Modify {
                id,
                name,
                r#type,
                instructions,
                notes,
            } => {
                let mut query = PropagationProcedure::update_by_id(id);
                if let Some(name) = name {
                    query = query.name(name);
                }
                if let Some(t) = r#type {
                    query = query.r#type(t);
                }
                if let Some(instructions) = instructions {
                    query = query.instructions(instructions);
                }
                if let Some(notes) = notes {
                    query = query.notes(notes);
                }
                query.exec(db).await?;
                load_and_display_propagation_details(db, format, id).await?;
            }
            PropagationCommands::Remove { id, assumeyes } => {
                if *assumeyes
                    || {
                        let procedure: PropagationProcedureDetails =
                            PropagationProcedure::filter_by_id(id)
                                .include(PropagationProcedure::fields().taxa().taxon())
                                .one()
                                .exec(db)
                                .await?
                                .into();

                        println!(
                            "{}",
                            PropagationProcedureDetailView::new(&procedure).render()?
                        );
                        confirm().with_prompt(
                        "Are you sure you wish to remove this Propagation procedure? It will remove all related steps",
                    )
                    .default(false)
                    .interact()?
                    }
                {
                    PropagationProcedure::delete_by_id(db, id).await?;
                    println!("Removed propagation procedure {id}");
                }
            }
            PropagationCommands::Import { path } => {
                #[derive(Debug, Deserialize)]
                struct PropagationInfo {
                    pub name: String,
                    pub instructions: String,
                    pub notes: Option<String>,
                    pub r#type: ProcedureType,
                }
                let procedures: Vec<PropagationInfo> =
                    serde_yaml::from_reader(std::fs::File::open(path)?)?;
                for p in procedures {
                    PropagationProcedure::create()
                        .name(p.name)
                        .instructions(p.instructions)
                        .notes(p.notes)
                        .r#type(p.r#type)
                        .exec(db)
                        .await?;
                }
            }
            PropagationCommands::AddCitation {
                propagation_id,
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
                PropagationProcedureCitation::create()
                    .citation_id(citation.id)
                    .propagation_id(propagation_id)
                    .exec(db)
                    .await?;
                load_and_display_propagation_details(db, format, propagation_id).await?;
            }
            PropagationCommands::RemoveCitation {
                propagation_id,
                citation_id,
                assumeyes,
            } => {
                if *assumeyes || {
                    let pc: CitationDetails =
                        PropagationProcedureCitation::filter_by_citation_id_and_propagation_id(
                            citation_id,
                            propagation_id,
                        )
                        .include(PropagationProcedureCitation::fields().citation())
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
                    PropagationProcedureCitation::delete_by_citation_id_and_propagation_id(
                        db,
                        citation_id,
                        propagation_id,
                    )
                    .await?;
                    let citation = Citation::filter_by_id(citation_id)
                        .include(Citation::fields().propagation_procedures())
                        .include(Citation::fields().taxon_propagation_procedures())
                        .include(Citation::fields().cleaning_procedures())
                        .include(Citation::fields().taxon_cleaning_procedures())
                        .one()
                        .exec(db)
                        .await?;
                    // if the citation is no longer rused, remove it from the database
                    if citation.propagation_procedures.get().is_empty()
                        && citation.taxon_propagation_procedures.get().is_empty()
                        && citation.cleaning_procedures.get().is_empty()
                        && citation.taxon_cleaning_procedures.get().is_empty()
                    {
                        Citation::delete_by_id(db, citation_id).await?;
                    }
                    load_and_display_propagation_details(db, format, propagation_id).await?;
                }
            }
        }
        Ok(())
    }
}

async fn load_and_display_propagation_details(
    db: &mut Db,
    format: OutputFormat,
    id: &u64,
) -> Result<(), anyhow::Error> {
    let procedure: PropagationProcedureDetails = PropagationProcedure::filter_by_id(id)
        .include(PropagationProcedure::fields().taxa().taxon())
        .include(PropagationProcedure::fields().citations())
        .one()
        .exec(db)
        .await?
        .into();
    let output = match format {
        OutputFormat::Text => PropagationProcedureDetailView::new(&procedure).render()?,
        OutputFormat::Json => JsonView::new(&procedure).render()?,
        OutputFormat::Yaml => YamlView::new(&procedure).render()?,
    };
    println!("{output}");
    Ok(())
}
