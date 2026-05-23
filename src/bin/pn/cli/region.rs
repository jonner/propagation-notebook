use std::path::PathBuf;

use propagation_notebook::region::{ConservationStatus, Origin, WetlandIndicator};

#[derive(clap::Parser, Debug)]
pub struct RegionalTaxonId {
    #[arg(short, long, help = "ID of a region")]
    pub region_id: u64,
    #[arg(short, long, help = "ID of a taxon")]
    pub taxon_id: u64,
}
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

#[derive(clap::Args, Debug)]
pub struct RegionalTaxonProperties {
    #[arg(short, long, help = "Origin of the taxon vis-a-vis this region")]
    pub origin: Option<Origin>,
    #[arg(
        long,
        help = "Coefficient of conservatism (0-10) for the species in this region"
    )]
    pub c_value: Option<u64>,
    #[arg(
        short,
        long,
        help = "Conservation status for the species in the given region"
    )]
    pub conservation_status: Option<ConservationStatus>,
    #[arg(
        short,
        long,
        help = "Whether the species is a wetland indicator in the given region"
    )]
    pub wetland_indicator: Option<WetlandIndicator>,
    // harvest phenology
    #[arg(
        long,
        help = "Start of the harvest window for the species in the given region"
    )]
    pub harvest_start: Option<jiff::civil::Date>,
    #[arg(
        long,
        help = "End of the harvest window for the species in the given region"
    )]
    pub harvest_end: Option<jiff::civil::Date>,
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
        #[arg(long, help = "Free-form notes about the region")]
        notes: Option<String>,
    },
    #[command(about = "Import a new region to the database")]
    Import {
        #[arg(help = "A path to a yaml file describing a region")]
        path: PathBuf,
    },
    #[command(about = "Modify information about a region", group(clap::ArgGroup::new("modify_fields").args(["name", "bounds_string", "bounds_file", "notes"]).required(true).multiple(true)))]
    Modify {
        id: u64,
        #[command(flatten)]
        bounds: BoundsArg,
        #[arg(short, long, help = "Specify a new name for the region")]
        name: Option<String>,
        #[arg(long, help = "Set notes for a region")]
        notes: Option<String>,
    },
    #[command(about = "Remove a region from the database")]
    Remove { id: u64 },
    #[command(about = "Manage taxa for a region")]
    Taxa {
        #[command(subcommand)]
        command: RegionTaxaCommands,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum RegionTaxaCommands {
    #[command(about = "Print a list of taxa for a region")]
    List {
        #[arg(short, long, help = "ID of a region")]
        region_id: u64,
    },
    #[command(about = "Add a taxon to a region")]
    Add {
        #[command(flatten)]
        id: RegionalTaxonId,
        #[command(flatten)]
        props: RegionalTaxonProperties,
    },
    #[command(about = "Modify information about a taxon within a region", group(clap::ArgGroup::new("modify_taxon_fields").args(["origin", "c_value", "conservation_status", "wetland_indicator", "harvest_start", "harvest_end"]).required(true).multiple(false)))]
    Modify {
        #[command(flatten)]
        id: RegionalTaxonId,
        #[command(flatten)]
        props: RegionalTaxonProperties,
    },
    #[command(about = "Remove a taxon from a region")]
    Remove {
        #[command(flatten)]
        id: RegionalTaxonId,
    },
}
