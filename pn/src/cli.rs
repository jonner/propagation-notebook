use crate::cli::propagation::PropagationCommands;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Debug, clap::Parser)]
#[command(version)]
pub struct Options {
    #[command(subcommand)]
    pub command: MainCommand,
    #[arg(long="fmt", global = true, value_enum, default_value_t = OutputFormat::Text, help = "Choose output format")]
    pub format: OutputFormat,
}

#[derive(Debug, clap::Subcommand)]
pub enum MainCommand {
    #[command(about = "Taxonomy-related commands", aliases=["taxon", "t", "species"])]
    Taxa {
        #[command(subcommand)]
        command: taxa::TaxonCommands,
    },
    #[command(about = "Region-related commands", aliases=["region", "r"])]
    Regions {
        #[command(subcommand)]
        command: region::RegionCommands,
    },
    #[command(about = "Seed propagation information", alias = "germination")]
    Propagation {
        #[command(subcommand)]
        command: PropagationCommands,
    },
    #[command(about = "Import data into the databse")]
    Import {
        #[command(subcommand)]
        command: import::ImportCommands,
    },
    #[command(about = "Initialize the database")]
    Init,
}

pub mod citation;
pub mod import;
pub mod propagation;
pub mod region;
pub mod taxa;
