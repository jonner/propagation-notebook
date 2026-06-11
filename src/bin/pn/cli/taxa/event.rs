use propagation_notebook::collecting::HarvestEvent;
use toasty::Db;

use crate::style;

#[derive(Debug, clap::Subcommand)]
pub enum TaxonHarvestEventCommands {
    #[command(about = "Print a list of all harvest events")]
    List,
    #[command(about = "Add a harvest event")]
    Add {
        #[arg(short, long, help = "The date of the harvest event")]
        date: jiff::civil::Date,
        #[arg(short, long, help = "free text harvest event")]
        notes: Option<String>,
        #[arg(short, long, help = "A location ID")]
        location_id: u64,
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
        }
        Ok(())
    }
}
