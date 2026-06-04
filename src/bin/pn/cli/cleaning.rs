#[derive(Debug, clap::Subcommand)]
pub enum CleaningCommands {
    #[command(about = "List all seed cleaning procedures")]
    List,
    #[command(about = "Show detailed information about a seed cleaning procedure")]
    Show { id: u64 },
    #[command(about = "Add a new seed cleaning procedure")]
    Add {
        #[arg(short, long, help = "A name for the procedure")]
        name: String,
        #[arg(short, long, help = "A name for the procedure")]
        instructions: String,
        #[arg(long, help = "General notes about the procedure")]
        notes: Option<String>,
    },
    #[command(about = "Modify a seed cleaning procedure", group(clap::ArgGroup::new("cleaning_props").args(["name", "instructions", "notes"]).required(true).multiple(false)))]
    Modify {
        id: u64,
        #[arg(short, long, help = "A name for the procedure")]
        instructions: Option<String>,
        #[arg(short, long, help = "A name for the procedure")]
        name: Option<String>,
        #[arg(long, help = "General notes about the procedure")]
        notes: Option<String>,
    },
    #[command(about = "Remove a seed cleaning procedure")]
    Remove {
        id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}
