use libpropagation::collecting::{
    CleaningProcedure,
    dto::{CleaningProcedureCompact, CleaningProcedureDetails},
};
use toasty::Db;

use crate::{
    cli::OutputFormat,
    views::cleaning::{CleaningProcedureDetailsView, CleaningProcedureListView},
};

#[derive(Debug, clap::Subcommand)]
pub enum CleaningCommands {
    #[command(about = "List all seed cleaning procedures")]
    List,
    #[command(about = "Show detailed information about a seed cleaning procedure")]
    Show { id: u64 },
    #[command(about = "Add a new seed cleaning procedure")]
    Add {
        #[arg(short, long, help = "A name for the procedure")]
        name: String,
        #[arg(short, long, help = "A name for the procedure")]
        instructions: String,
        #[arg(long, help = "General notes about the procedure")]
        notes: Option<String>,
        #[arg(long, help = "A citation for the procedure")]
        citation: Option<String>,
    },
    #[command(about = "Modify a seed cleaning procedure", group(clap::ArgGroup::new("cleaning_props").args(["name", "instructions", "notes", "citation"]).required(true).multiple(false)))]
    Modify {
        id: u64,
        #[arg(short, long, help = "A name for the procedure")]
        instructions: Option<String>,
        #[arg(short, long, help = "A name for the procedure")]
        name: Option<String>,
        #[arg(long, help = "General notes about the procedure")]
        notes: Option<String>,
        #[arg(long, help = "A citation for the procedure")]
        citation: Option<String>,
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
}

impl CleaningCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            CleaningCommands::List => {
                let items: Vec<CleaningProcedureCompact> = CleaningProcedure::all()
                    .include(CleaningProcedure::fields().taxon_links().taxon())
                    .exec(db)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect();
                let output = match format {
                    OutputFormat::Text => CleaningProcedureListView::new(&items).render()?,
                    OutputFormat::Json => todo!(),
                    OutputFormat::Yaml => todo!(),
                };
                println!("{output}");
            }
            CleaningCommands::Show { id } => {
                let procedure: CleaningProcedureDetails = CleaningProcedure::filter_by_id(id)
                    .include(CleaningProcedure::fields().taxon_links().taxon())
                    .one()
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => CleaningProcedureDetailsView::new(&procedure).render()?,
                    OutputFormat::Json => todo!(),
                    OutputFormat::Yaml => todo!(),
                };
                println!("{output}");
            }
            CleaningCommands::Add {
                name,
                instructions,
                notes,
                citation,
            } => {
                let item: CleaningProcedureDetails = CleaningProcedure::create()
                    .name(name)
                    .instructions(instructions)
                    .notes(notes)
                    .citation(citation)
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => CleaningProcedureDetailsView::new(&item).render()?,
                    OutputFormat::Json => todo!(),
                    OutputFormat::Yaml => todo!(),
                };
                println!("{output}");
            }
            CleaningCommands::Remove { id, assumeyes } => {
                if *assumeyes || {
                    let item: CleaningProcedureDetails = CleaningProcedure::filter_by_id(id)
                        .include(CleaningProcedure::fields().taxon_links().taxon())
                        .one()
                        .exec(db)
                        .await?
                        .into();
                    println!("{}", CleaningProcedureDetailsView::new(&item).render()?);
                    inquire::Confirm::new(&format!(
                        "Are you sure you wish to remove cleaning procedure {id}?"
                    ))
                    .with_default(false)
                    .with_help_message(&format!("It is used by {} taxa", item.taxa.len()))
                    .prompt()?
                } {
                    CleaningProcedure::delete_by_id(db, id).await?;
                    println!("Removed cleaning procedure {id}");
                }
            }
            CleaningCommands::Modify {
                id,
                name,
                instructions,
                notes,
                citation,
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
                if let Some(citation) = citation {
                    query = query.citation(citation);
                }
                query.exec(db).await?;
                let item: CleaningProcedureDetails = CleaningProcedure::filter_by_id(id)
                    .include(CleaningProcedure::fields().taxon_links().taxon())
                    .one()
                    .exec(db)
                    .await?
                    .into();
                println!("{}", CleaningProcedureDetailsView::new(&item).render()?);
            }
        }
        Ok(())
    }
}
