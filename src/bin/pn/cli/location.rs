use propagation_notebook::collecting::Location;
use toasty::Db;

use crate::style;

#[derive(Debug, clap::Subcommand)]
pub enum LocationCommands {
    #[command(about = "Print a list of locations")]
    List,
    #[command(about = "Show details of a location")]
    Show {
        #[arg(help = "A location ID")]
        id: u64,
    },
    #[command(about = "Add a new location")]
    Add {
        #[arg(short, long, help = "A short name for the location")]
        name: String,
        #[arg(short = 'y', long, help = "A latitude coordinate")]
        latitude: f32,
        #[arg(short = 'x', long, help = "A longitude coordinate")]
        longitude: f32,
    },
    #[command(about = "Add a new location", group(clap::ArgGroup::new("modify_args").args(["name", "latitude", "longitude"]).required(true)))]
    Modify {
        #[arg(help = "A location ID")]
        id: u64,
        #[arg(short, long, help = "A short name for the location")]
        name: Option<String>,
        #[arg(short = 'y', long, help = "A latitude coordinate")]
        latitude: Option<f32>,
        #[arg(short = 'x', long, help = "A longitude coordinate")]
        longitude: Option<f32>,
    },
    #[command(about = "Remove a location")]
    Remove {
        #[arg(help = "A location ID")]
        id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl LocationCommands {
    pub async fn run(&self, db: &mut Db) -> anyhow::Result<()> {
        match self {
            LocationCommands::List => {
                let locations = Location::all().exec(db).await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Name"]);
                for location in locations {
                    tbuilder.push_record([location.id.to_string(), location.name])
                }
                println!("{}", tbuilder.build().with(style::BasicTable));
            }
            LocationCommands::Show { id } => {
                let loc = Location::get_by_id(db, id).await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &loc.id.to_string()]);
                tbuilder.push_record(["Name", &loc.name]);
                tbuilder.push_record(["Latitude", &loc.latitude.to_string()]);
                tbuilder.push_record(["Longitude", &loc.longitude.to_string()]);
                println!("{}", tbuilder.build().with(style::DetailTable));
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
            LocationCommands::Modify {
                id,
                name,
                latitude,
                longitude,
            } => {
                let mut query = Location::update_by_id(id);
                if let Some(name) = name {
                    query = query.name(name);
                }
                if let Some(latitude) = latitude {
                    query = query.latitude(latitude);
                }
                if let Some(longitude) = longitude {
                    query = query.longitude(longitude);
                }
                query.exec(db).await?;
                println!("Modified location {}", id);
            }
            LocationCommands::Remove { id, assumeyes } => {
                if *assumeyes
                    || inquire::Confirm::new("Are you sure you wish to delete this location?")
                        .with_default(false)
                        .with_help_message("All associated data will be deleted")
                        .prompt()?
                {
                    Location::delete_by_id(db, id).await?;
                    println!("Deleted location {id} from the database");
                }
            }
        }
        Ok(())
    }
}
