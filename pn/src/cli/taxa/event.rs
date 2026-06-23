use libpropagation::collecting::HarvestEvent;
use toasty::Db;

use crate::style;

#[derive(Debug, clap::Subcommand)]
pub enum TaxonHarvestEventCommands {
    #[command(about = "Print a list of all harvest events")]
    List,
    #[command(about = "Show details of a harvest event")]
    Show {
        #[arg(help = "A harvest event ID")]
        id: u64,
    },
    #[command(about = "Add a harvest event")]
    Add {
        #[arg(short, long, help = "The date of the harvest event")]
        date: jiff::civil::Date,
        #[arg(short, long, help = "free text harvest event")]
        notes: Option<String>,
        #[arg(short, long, help = "A location ID")]
        location_id: u64,
    },
    #[command(about = "Modify a harvest event", group(clap::ArgGroup::new("modify_args").args(["date", "notes", "location_id"]).required(true)))]
    Modify {
        #[arg(help = "A harvest event ID")]
        id: u64,
        #[arg(short, long, help = "The date of the harvest event")]
        date: Option<jiff::civil::Date>,
        #[arg(short, long, help = "free text harvest event")]
        notes: Option<String>,
        #[arg(short, long, help = "A location ID")]
        location_id: Option<u64>,
    },
    Remove {
        #[arg(help = "A harvest event ID")]
        id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl TaxonHarvestEventCommands {
    pub async fn run(&self, db: &mut Db, taxon_id: u64) -> anyhow::Result<()> {
        match self {
            TaxonHarvestEventCommands::List => {
                let events = HarvestEvent::filter_by_taxon_id(taxon_id)
                    .include(HarvestEvent::fields().location())
                    .exec(db)
                    .await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Date", "Location"]);
                for event in events {
                    tbuilder.push_record([
                        event.id.to_string(),
                        event.date.to_string(),
                        event.location.get().reference(),
                    ])
                }
                println!("{}", tbuilder.build().with(style::ListTable));
            }
            TaxonHarvestEventCommands::Show { id } => {
                let event = HarvestEvent::filter_by_id(id)
                    .include(HarvestEvent::fields().taxon())
                    .include(HarvestEvent::fields().location())
                    .one()
                    .exec(db)
                    .await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &event.id.to_string()]);
                tbuilder.push_record(["Date", &event.date.to_string()]);
                tbuilder.push_record(["Taxon", &event.taxon.get().reference()]);
                tbuilder.push_record(["Location", &event.location.get().reference()]);
                tbuilder.push_record(["Notes", event.notes.as_deref().unwrap_or("-")]);
                println!("{}", tbuilder.build().with(style::DetailTable));
            }
            TaxonHarvestEventCommands::Add {
                date,
                notes,
                location_id,
            } => {
                let event = HarvestEvent::create()
                    .date(date)
                    .notes(notes)
                    .location_id(location_id)
                    .taxon_id(taxon_id)
                    .exec(db)
                    .await?;
                println!("Added event {}", event.id);
            }
            TaxonHarvestEventCommands::Modify {
                id,
                date,
                notes,
                location_id,
            } => {
                let mut query = HarvestEvent::update_by_id(id);
                if let Some(date) = date {
                    query = query.date(date);
                }
                if let Some(notes) = notes {
                    query = query.notes(notes);
                }
                if let Some(location_id) = location_id {
                    query = query.location_id(location_id);
                }
                query.exec(db).await?;
                println!("Modified event {}", id);
            }
            TaxonHarvestEventCommands::Remove { id, assumeyes } => {
                if *assumeyes
                    || inquire::Confirm::new("Are you sure you want to remove this harvest event?")
                        .with_default(false)
                        .prompt()?
                {
                    HarvestEvent::delete_by_id(db, id).await?;
                    println!("Removed harvest event {id} from the database");
                }
            }
        }
        Ok(())
    }
}
