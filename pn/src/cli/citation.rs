#[derive(Debug, Clone, clap::Subcommand)]
pub enum CitationCommands {
    List,
    Show {
        #[arg(help = "A Citation ID")]
        id: u64,
    },
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
