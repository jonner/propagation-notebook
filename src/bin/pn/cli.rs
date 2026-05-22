use crate::cli::{cleaning::CleaningCommands, propagation::PropagationCommands};

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
    #[command(about = "Seed propagation information")]
    Propagation {
        #[command(subcommand)]
        command: PropagationCommands,
    },
}

pub mod cleaning;
pub mod collecting;
pub mod propagation;
pub mod region;
pub mod taxa;
