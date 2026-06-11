use propagation_notebook::collecting::Location;
use toasty::Db;

use crate::style;

#[derive(Debug, clap::Subcommand)]
pub enum LocationCommands {
    #[command(about = "Print a list of locations")]
    List,
    #[command(about = "Add a new location")]
    Add {
        #[arg(short, long, help = "A short name for the location")]
        name: String,
        #[arg(short = 'y', long, help = "A latitude coordinate")]
        latitude: f32,
        #[arg(short = 'x', long, help = "A longitude coordinate")]
        longitude: f32,
    },
}

impl LocationCommands {
    pub async fn run(&self, db: &mut Db) -> anyhow::Result<()> {
        match self {
            LocationCommands::List => {
                let locations = Location::all().exec(db).await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Name", "Latitude", "Longitude"]);
                for location in locations {
                    tbuilder.push_record([
                        location.id.to_string(),
                        location.name,
                        location.latitude.to_string(),
                        location.longitude.to_string(),
                    ])
                }
                println!("{}", tbuilder.build().with(style::BasicTable));
            }
            LocationCommands::Add {
                name,
                latitude,
                longitude,
            } => {
                let loc = Location::create()
                    .name(name)
                    .latitude(latitude)
                    .longitude(longitude)
                    .exec(db)
                    .await?;
                println!("Added location {}", loc.id);
            }
        }
        Ok(())
    }
}
