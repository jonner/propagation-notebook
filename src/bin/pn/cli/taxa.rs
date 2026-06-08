#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum TaxonomicAuthority {
    Itis,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCommands {
    #[command(about = "Print a list of all taxa")]
    List {
        #[arg(short, long, help = "Show only taxa in the specified region")]
        region_id: Option<u64>,
    },
    #[command(about = "Show detailed information about a Taxon")]
    Show { id: u64 },
    #[command(about = "Search for a taxon")]
    Search { search_string: String },
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
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    #[command(about = "Manage collecting information for a taxon")]
    Collecting {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: TaxonCollectingCommands,
    },
    #[command(about = "Manage cleaning information for a taxon")]
    Cleaning {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: TaxonCleaningCommands,
    },
    #[command(about = "Manage cleaning information for a taxon")]
    Propagation {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: TaxonPropagationCommands,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCleaningCommands {
    #[command(about = "Show all seed cleaning procedures for a taxon")]
    List,
    #[command(about = "Show all seed cleaning procedures for a taxon")]
    Show {
        #[arg(short, long, help = "A cleaning procedure ID")]
        procedure_id: u64,
    },
    #[command(about = "Associate a taxon with a seed cleaning procedure")]
    Add {
        #[arg(short, long, help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(short, long, help = "Taxon-specific notes for this procedure")]
        notes: Option<String>,
    },
    #[command(about = "Modify taxon-specific information seed cleaning information")]
    Modify {
        #[arg(short, long, help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(short, long, help = "Taxon-specific notes for this procedure")]
        notes: Option<String>,
    },
    #[command(about = "Remove a cleaning procedure from the specified taxon")]
    Remove {
        #[arg(short, long, help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCollectingCommands {
    #[command(about = "Show seed collecting information")]
    Show,
    #[command(about = "Add new seed collecting information for a taxon")]
    Add {
        #[arg(
            short,
            long,
            help = "What to look for to determine if the seed is ready for collecting"
        )]
        ripening_indicators: Option<String>,
        #[arg(short, long, help = "Instructions for storing the seed")]
        storage_conditions: Option<String>,
        #[arg(
            short = 'l',
            long,
            help = "How long the seed will stay viable in storage"
        )]
        storage_life: Option<String>,
    },
    #[command(about = "Modify seed collecting information for a taxon", group(clap::ArgGroup::new("modify_props").args(["ripening_indicators", "storage_conditions", "storage_life"]).required(true).multiple(false)))]
    Modify {
        #[arg(
            short,
            long,
            help = "What to look for to determine if the seed is ready for collecting"
        )]
        ripening_indicators: Option<String>,
        #[arg(short, long, help = "Instructions for storing the seed")]
        storage_conditions: Option<String>,
        #[arg(
            short = 'l',
            long,
            help = "How long the seed will stay viable in storage"
        )]
        storage_life: Option<String>,
    },
    #[command(about = "Remove seed collecting information")]
    Remove {
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum TaxonPropagationCommands {
    #[command(about = "List seed propagation protocols for the taxon")]
    List,
    #[command(about = "Show seed propagation information for the taxon")]
    Show {
        #[arg(
            short,
            long,
            help = "An ID of a propagation protocol ID assigned to this taxon"
        )]
        protocol_id: u64,
    },
    #[command(about = "Assign a new seed propagation protocol to a taxon")]
    Add {
        #[arg(short, long, help = "An ID of a propagation protocol ID")]
        protocol_id: u64,
        #[arg(
            short,
            long,
            help = "Confidence level in this propagation protocol (0-10)",
            value_parser = clap::value_parser!(u8).range(0..=10)
        )]
        confidence: Option<u8>,
        #[arg(
            short,
            long,
            help = "Taxon-specific notes for this propagation protocol"
        )]
        notes: Option<String>,
    },
    #[command(about = "Modify propagation information for a taxon", group(clap::ArgGroup::new("modify_props").args(["confidence", "notes"]).required(true).multiple(false)))]
    Modify {
        #[arg(short, long, help = "A propagation protocol ID assigned to this taxon")]
        protocol_id: u64,
        #[arg(
            short,
            long,
            help = "Confidence level in this propagation protocol (0-10)",
            value_parser = clap::value_parser!(u8).range(0..=10)
        )]
        confidence: Option<u8>,
        #[arg(
            short,
            long,
            help = "Taxon-specific notes for this propagation protocol"
        )]
        notes: Option<String>,
    },
    #[command(about = "Remove propagation information from the taxon")]
    Remove {
        #[arg(short, long, help = "A propagation protocol ID assigned to this taxon")]
        protocol_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}
