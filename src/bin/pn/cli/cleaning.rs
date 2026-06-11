use propagation_notebook::collecting::CleaningProcedure;
use toasty::Db;

use crate::{style, util::join_or_default};

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
    },
    #[command(about = "Modify a seed cleaning procedure", group(clap::ArgGroup::new("cleaning_props").args(["name", "instructions", "notes"]).required(true).multiple(false)))]
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
}

impl CleaningCommands {
    pub async fn run(&self, db: &mut Db) -> anyhow::Result<()> {
        match self {
            CleaningCommands::List => {
                let items = CleaningProcedure::all()
                    .include(CleaningProcedure::fields().taxon_links().taxon())
                    .exec(db)
                    .await?;
                let nitems = items.len();
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Name", "Taxa"]);
                for item in items {
                    tbuilder.push_record([
                        item.id.to_string(),
                        item.name,
                        item.taxon_links.get().len().to_string(),
                    ])
                }
                println!("{}", tbuilder.build().with(style::ListTable));
                println!("\n{nitems} found");
            }
            CleaningCommands::Show { id } => {
                let procedure = CleaningProcedure::filter_by_id(id)
                    .include(CleaningProcedure::fields().taxon_links().taxon())
                    .one()
                    .exec(db)
                    .await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &procedure.id.to_string()]);
                tbuilder.push_record(["Name", &procedure.name]);
                tbuilder.push_record(["Notes", &procedure.notes.unwrap_or_else(|| "-".into())]);
                tbuilder.push_record([
                    "Taxa",
                    &join_or_default(procedure.taxon_links.get(), "-", |v| {
                        v.taxon.get().reference()
                    }),
                ]);
                tbuilder.push_record(["Instructions", &procedure.instructions]);
                println!("{}", tbuilder.build().with(style::ListTable));
            }
            CleaningCommands::Add {
                name,
                instructions,
                notes,
            } => {
                let item = CleaningProcedure::create()
                    .name(name)
                    .instructions(instructions)
                    .notes(notes)
                    .exec(db)
                    .await?;
                println!("Added new procedure {}", item.id);
            }
            CleaningCommands::Remove { id, assumeyes } => {
                let item = CleaningProcedure::filter_by_id(id)
                    .include(CleaningProcedure::fields().taxon_links())
                    .one()
                    .exec(db)
                    .await?;
                if *assumeyes
                    || inquire::Confirm::new(&format!(
                        "Are you sure you wish to remove cleaning procedure {id}?"
                    ))
                    .with_default(false)
                    .with_help_message(&format!(
                        "It is used by {} taxa",
                        item.taxon_links.get().len()
                    ))
                    .prompt()?
                {
                    CleaningProcedure::delete_by_id(db, id).await?;
                    println!("Removed cleaning procedure {id}");
                }
            }
            CleaningCommands::Modify {
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
                println!("Modified cleaning procedure {id}");
            }
        }
        Ok(())
    }
}
