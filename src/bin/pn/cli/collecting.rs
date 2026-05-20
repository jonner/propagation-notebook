#[derive(Debug, clap::Subcommand)]
pub enum CollectingCommands {
    #[command(about = "List all seed collection information")]
    List,
    #[command(about = "Show the specified seed collection information ",
  group(clap::ArgGroup::new("id_fields").args(["id", "taxon_id"]).required(true).multiple(false)))]
    Show {
        #[arg(short, long, help = "ID of seed collection data")]
        id: Option<u64>,
        #[arg(short, long, help = "ID of a Taxon")]
        taxon_id: Option<u64>,
    },
    #[command(about = "Add new seed collection information for a taxon")]
    Add {
        #[arg(short, long, help = "ID of a Taxon")]
        taxon_id: u64,
        #[arg(
            short,
            long,
            help = "What to look for to determine if the seed is ready for collection"
        )]
        ripening_indicators: String,
        #[arg(short, long, help = "Instructions for storing the seed")]
        storage: Option<String>,
    },
}
