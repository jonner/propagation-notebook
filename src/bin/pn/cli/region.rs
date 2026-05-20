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
    #[command(about = "Remove a region from the database")]
    Remove { id: u64 },
    #[command(about = "Print a list of taxa for a region")]
    ListTaxa {
        #[arg(short, long, help = "ID of a region")]
        region_id: u64,
    },
    #[command(about = "Show detailed information about the status of a taxon within a region")]
    ShowTaxon {
        #[command(flatten)]
        id: RegionalTaxonId,
    },
    #[command(about = "Add a taxon to a region")]
    AddTaxon {
        #[command(flatten)]
        id: RegionalTaxonId,
        #[arg(short, long, help = "Origin of the taxon vis-a-vis this region")]
        origin: Option<Origin>,
        #[arg(
            long,
            help = "Coefficient of conservatism (0-10) for the species in this region"
        )]
        c_value: Option<u64>,
        #[arg(
            short,
            long,
            help = "Conservation status for the species in the given region"
        )]
        conservation_status: Option<ConservationStatus>,
        #[arg(
            short,
            long,
            help = "Whether the species is a wetland indicator in the given region"
        )]
        wetland_indicator: Option<WetlandIndicator>,
        // harvest phenology
        #[arg(
            long,
            help = "Start of the harvest window for the species in the given region"
        )]
        harvest_start: Option<jiff::civil::Date>,
        #[arg(
            long,
            help = "End of the harvest window for the species in the given region"
        )]
        harvest_end: Option<jiff::civil::Date>,
    },
    #[command(about = "Remove a taxon from a region")]
    RemoveTaxon {
        #[command(flatten)]
        id: RegionalTaxonId,
    },
}
