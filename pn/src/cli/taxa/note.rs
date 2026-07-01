use libpropagation::taxonomy::TaxonNote;
use toasty::Db;

use crate::{
    cli::OutputFormat,
    views::{
        JsonView, YamlView,
        taxa::{TaxonNoteDetailsView, TaxonNotesListView},
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum TaxonNoteCommands {
    #[command(about = "List notes for the taxon")]
    List,
    #[command(about = "Show a note for the taxon")]
    Show {
        #[arg(help = "A note ID")]
        note_id: u64,
    },
    #[command(about = "Add a new note to a taxon")]
    Add {
        #[arg(help = "A note")]
        text: String,
    },
    #[command(about = "Modify a note for a taxon")]
    Modify {
        #[arg(help = "A propagation protocol ID assigned to this taxon")]
        note_id: u64,
        #[arg(help = "A note")]
        text: String,
    },
    #[command(about = "Remove a note from the taxon")]
    Remove {
        #[arg(help = "A note ID")]
        note_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl TaxonNoteCommands {
    pub async fn run(
        &self,
        db: &mut Db,
        taxon_id: u64,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            TaxonNoteCommands::List => {
                let notes = TaxonNote::filter_by_taxon_id(taxon_id).exec(db).await?;
                let output = match format {
                    OutputFormat::Text => TaxonNotesListView::new(&notes).render()?,
                    OutputFormat::Json => JsonView::new(&notes).render()?,
                    OutputFormat::Yaml => YamlView::new(&notes).render()?,
                };
                println!("{output}");
            }
            TaxonNoteCommands::Show { note_id } => {
                let note = TaxonNote::filter_by_id(note_id)
                    .include(TaxonNote::fields().taxon())
                    .one()
                    .exec(db)
                    .await?;
                let output = match format {
                    OutputFormat::Text => TaxonNoteDetailsView::new(&note).render()?,
                    OutputFormat::Json => JsonView::new(&note).render()?,
                    OutputFormat::Yaml => YamlView::new(&note).render()?,
                };
                println!("{output}");
            }
            TaxonNoteCommands::Add { text } => {
                let note = TaxonNote::create()
                    .taxon_id(taxon_id)
                    .text(text)
                    .exec(db)
                    .await?;
                let output = match format {
                    OutputFormat::Text => TaxonNoteDetailsView::new(&note).render()?,
                    OutputFormat::Json => JsonView::new(&note).render()?,
                    OutputFormat::Yaml => YamlView::new(&note).render()?,
                };
                println!("{output}");
            }
            TaxonNoteCommands::Modify { note_id, text } => {
                TaxonNote::update_by_id(note_id).text(text).exec(db).await?;
                println!("Updated note {note_id}")
            }
            TaxonNoteCommands::Remove { note_id, assumeyes } => {
                let note = TaxonNote::get_by_id(db, note_id).await?;
                if *assumeyes || {
                    println!("{}", TaxonNoteDetailsView::new(&note).render()?);
                    inquire::Confirm::new(&format!(
                        "Are you sure you wish to remove note {note_id}?"
                    ))
                    .with_default(false)
                    .prompt()?
                } {
                    {
                        TaxonNote::delete_by_id(db, note_id).await?;
                        println!("Removed note {note_id}");
                    }
                }
            }
        }
        Ok(())
    }
}
