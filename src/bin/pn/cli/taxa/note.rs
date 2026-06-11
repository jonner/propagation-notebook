use propagation_notebook::taxonomy::TaxonNote;
use toasty::Db;

use crate::style;

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
    pub async fn run(&self, db: &mut Db, taxon_id: u64) -> anyhow::Result<()> {
        match self {
            TaxonNoteCommands::List => {
                let notes = TaxonNote::filter_by_taxon_id(taxon_id).exec(db).await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Note"]);
                for note in notes {
                    tbuilder.push_record([note.id.to_string(), note.text]);
                }
                println!("{}", tbuilder.build().with(style::ListTable));
            }
            TaxonNoteCommands::Show { note_id } => {
                let note = TaxonNote::get_by_id(db, note_id).await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &note.id.to_string()]);
                tbuilder.push_record(["Text", &note.text]);
                tbuilder.push_record(["Created", &note.created_at.to_string()]);
                tbuilder.push_record(["Updated", &note.updated_at.to_string()]);
                println!("{}", tbuilder.build().with(style::DetailTable));
            }
            TaxonNoteCommands::Add { text } => {
                let note = TaxonNote::create()
                    .taxon_id(taxon_id)
                    .text(text)
                    .exec(db)
                    .await?;
                println!("added note {} to taxon {}", note.id, taxon_id);
            }
            TaxonNoteCommands::Modify { note_id, text } => {
                TaxonNote::update_by_id(note_id).text(text).exec(db).await?;
                println!("Updated note {note_id}")
            }
            TaxonNoteCommands::Remove { note_id, assumeyes } => {
                if *assumeyes
                    || inquire::Confirm::new("Are you sure you wish to remove this note?")
                        .with_default(false)
                        .prompt()?
                {
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
