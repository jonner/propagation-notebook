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
}
