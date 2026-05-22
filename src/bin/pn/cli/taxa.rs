#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum TaxonomicAuthority {
    Itis,
}

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
    #[command(about = "Import a new taxonomy for use with this tool")]
    Import {
        #[arg(help = "A URI to the external taxonomy database")]
        db_uri: String,
        #[arg(
            short,
            long,
            help = "The creator of the database",
            value_enum,
            default_value_t = TaxonomicAuthority::Itis
        )]
        authority: TaxonomicAuthority,
    },
}
