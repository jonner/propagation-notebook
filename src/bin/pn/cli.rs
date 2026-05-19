use propagation_notebook::region::{ConservationStatus, NativeStatus, WetlandIndicator};

#[derive(Debug, clap::Parser)]
pub struct Options {
    #[command(subcommand)]
    pub command: MainCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum MainCommand {
    #[command(about = "Taxonomy-related commands")]
    Taxa {
        #[command(subcommand)]
        command: TaxonCommands,
    },
    #[command(about = "Region-related commands")]
    Regions {
        #[command(subcommand)]
        command: RegionCommands,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCommands {
    #[command(about = "Search for a taxon")]
    Search { search_string: String },
    #[command(about = "Show detailed information about a Taxon")]
    Info { id: u64 },
    #[command(about = "Print a list of taxa that match a given set of filters")]
    List {
        #[arg(short, long, help = "Limit to taxa within the specified region")]
        region_id: Option<u64>,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum RegionCommands {
    #[command(about = "Print a list of regions")]
    List,
    #[command(about = "Add a new region to the database")]
    Add { region_name: String },
    #[command(about = "Add a taxon to the region")]
    AddSpecies {
        #[arg(short, long, help = "ID of a region in the database")]
        region_id: u64,
        #[arg(short, long, help = "ID of a taxon in the database")]
        taxon_id: u64,
        #[arg(
            short,
            long,
            help = "Native status of the given taxon within this region"
        )]
        native_status: Option<NativeStatus>,
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
}
