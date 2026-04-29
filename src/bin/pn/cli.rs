#[derive(Debug, clap::Parser)]
pub struct Options {
    #[command(subcommand)]
    pub command: MainCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum MainCommand {
    #[command(about = "Taxonomy-related commands")]
    Taxon {
        #[command(subcommand)]
        command: TaxonCommands,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCommands {
    #[command(about = "Search for a taxon")]
    Search { search_string: String },
    #[command(about = "Show detailed information about a Taxon")]
    Info { id: u64 },
}
