#[derive(Debug, Clone, clap::Subcommand)]
pub enum CitationCommands {
    #[command(about = "List all citations")]
    List,
    #[command(about = "Show citation details")]
    Show {
        #[arg(help = "A Citation ID")]
        id: u64,
    },
    #[command(about = "Link a citation to another object")]
    Link {
        #[arg(help = "A Citation ID")]
        id: u64,
    },
    #[command(about = "Add a new citation")]
    Add {
        #[arg(help = "Citation title")]
        title: String,
        #[arg(long, help = "A canonical URL for the citation")]
        url: Option<String>,
        #[arg(long, help = "The author being cited")]
        author: Option<String>,
        #[arg(long, help = "The date of the citation")]
        date: Option<jiff::civil::Date>,
    },
    #[command(about = "Remove a citation")]
    Remove {
        #[arg(help = "A citation ID")]
        citation_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}
