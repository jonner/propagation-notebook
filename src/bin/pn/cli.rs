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
}

#[derive(Debug, clap::Subcommand)]
pub enum RegionCommands {
    List,
    Add { region_name: String },
}
