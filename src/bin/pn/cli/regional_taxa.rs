use propagation_notebook::region::{ConservationStatus, Origin, WetlandIndicator};

#[derive(Debug, clap::Subcommand)]
pub enum RegionalTaxaCommands {
    #[command(about = "Print a list of taxa for a region")]
    List {
        #[arg(short, long, help = "ID of a region")]
        region_id: u64,
    },
    #[command(about = "Show detailed information about the status of a taxon within a region")]
    Show { id: u64 },
    #[command(about = "Add a taxon to a region")]
    Add {
        #[arg(short, long, help = "ID of a region")]
        region_id: u64,
        #[arg(short, long, help = "ID of a taxon")]
        taxon_id: u64,
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
    Remove { id: u64 },
}
