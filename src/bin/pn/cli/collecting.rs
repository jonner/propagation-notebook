#[derive(Debug, clap::Subcommand)]
pub enum CollectingCommands {
    #[command(about = "List all seed collecting information")]
    List,
    #[command(about = "Show seed collecting information",
  group(clap::ArgGroup::new("id_fields").args(["id", "taxon_id"]).required(true).multiple(false)))]
    Show {
        id: Option<u64>,
        #[arg(short, long, help = "ID of a Taxon")]
        taxon_id: Option<u64>,
    },
    #[command(about = "Add new seed collecting information for a taxon")]
    Add {
        #[arg(short, long, help = "ID of a Taxon")]
        taxon_id: u64,
        #[arg(
            short,
            long,
            help = "What to look for to determine if the seed is ready for collecting"
        )]
        ripening_indicators: String,
        #[arg(short, long, help = "Instructions for storing the seed")]
        storage: Option<String>,
    },
    #[command(about = "Add new seed collecting information for a taxon", group(clap::ArgGroup::new("modify_props").args(["ripening_indicators", "storage"]).required(true).multiple(false)))]
    Modify {
        id: u64,
        #[arg(
            short,
            long,
            help = "What to look for to determine if the seed is ready for collecting"
        )]
        ripening_indicators: Option<String>,
        #[arg(short, long, help = "Instructions for storing the seed")]
        storage: Option<String>,
    },
    #[command(about = "Remove seed collecting information")]
    Remove { id: u64 },
}
