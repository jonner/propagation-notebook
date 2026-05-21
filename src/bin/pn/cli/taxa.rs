#[derive(Debug, clap::Subcommand)]
pub enum TaxonCommands {
    #[command(about = "Print a list of all taxa")]
    List,
    #[command(about = "Show detailed information about a Taxon")]
    Show { id: u64 },
    #[command(about = "Search for a taxon")]
    Search { search_string: String },
    #[command(about = "Associate a taxon with a seed cleaning procedure")]
    SetCleaningProcedure {
        taxon_id: u64,
        #[arg(short, long, help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(
            short,
            long,
            help = "Taxon-specific notes for this procedure",
            conflicts_with = "remove"
        )]
        notes: Option<String>,
        #[arg(short, long, help = "Remove the assignment")]
        remove: bool,
    },
}
