use crate::cli::cleaning::CleaningCommands;

#[derive(Debug, clap::Parser)]
pub struct Options {
    #[command(subcommand)]
    pub command: MainCommand,
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
    #[command(about = "Seed collecting information")]
    Collecting {
        #[command(subcommand)]
        command: collecting::CollectingCommands,
    },
    #[command(about = "Seed cleaning information")]
    Cleaning {
        #[command(subcommand)]
        command: CleaningCommands,
    },
}

pub mod cleaning;
pub mod collecting;
pub mod region;
pub mod taxa;
