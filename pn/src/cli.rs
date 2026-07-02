use crate::cli::{cleaning::CleaningCommands, propagation::PropagationCommands};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Debug, clap::Parser)]
pub struct Options {
    #[command(subcommand)]
    pub command: MainCommand,
    #[arg(long="fmt", global = true, value_enum, default_value_t = OutputFormat::Text, help = "Choose output format")]
    pub format: OutputFormat,
}

#[derive(Debug, clap::Subcommand)]
pub enum MainCommand {
    #[command(about = "Taxonomy-related commands")]
    Taxa {
        #[command(subcommand)]
        command: taxa::TaxonCommands,
    },
    #[command(about = "Region-related commands")]
    Regions {
        #[command(subcommand)]
        command: region::RegionCommands,
    },
    #[command(about = "Seed cleaning information")]
    Cleaning {
        #[command(subcommand)]
        command: CleaningCommands,
    },
    #[command(about = "Seed propagation information")]
    Propagation {
        #[command(subcommand)]
        command: PropagationCommands,
    },
    #[command(about = "Initialize the database")]
    Init,
}

pub mod cleaning;
pub mod propagation;
pub mod region;
pub mod taxa;
