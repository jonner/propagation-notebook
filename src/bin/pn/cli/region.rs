use std::path::PathBuf;

#[derive(clap::Args, Debug)]
#[group(required = false, multiple = false)]
pub struct BoundsArg {
    #[arg(
        long,
        help = "path to a geojson file whose contents represent the bounds of the region",
        conflicts_with = "bounds_string"
    )]
    pub bounds_file: Option<PathBuf>,
    #[arg(
        short,
        long = "bounds",
        help = "geojson string representing the bounds of the region",
        conflicts_with = "bounds_file"
    )]
    pub bounds_string: Option<String>,
}

impl BoundsArg {
    pub async fn resolve(self) -> anyhow::Result<Option<String>> {
        match (self.bounds_string, self.bounds_file) {
            (Some(bounds_string), None) => Ok(Some(bounds_string)),
            (None, Some(bounds_file)) => Ok(Some(tokio::fs::read_to_string(bounds_file).await?)),
            (None, None) => Ok(None),
            _ => Err(anyhow::anyhow!(
                "Only one of 'bounds' or 'bounds_file' can be specified at the same time"
            )),
        }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum RegionCommands {
    #[command(about = "Print a list of regions")]
    List,
    #[command(about = "Show detailed information about a region")]
    Show { id: u64 },
    #[command(about = "Add a new region to the database")]
    Add {
        region_name: String,
        #[clap(flatten)]
        bounds: BoundsArg,
    },
    #[command(about = "Modify information about a region", group(clap::ArgGroup::new("modify_fields").args(["name", "bounds_string", "bounds_file"]).required(true).multiple(true)))]
    Modify {
        id: u64,
        #[command(flatten)]
        bounds: BoundsArg,
        #[arg(short, long, help = "Specify a new name for the region")]
        name: Option<String>,
    },
}
