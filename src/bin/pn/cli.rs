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
    #[command(about = "commands related to regional taxa lists")]
    RegionalTaxa {
        #[command(subcommand)]
        command: regional_taxa::RegionalTaxaCommands,
    },
    #[command(about = "Seed collecting information")]
    Collecting {
        #[command(subcommand)]
        command: collecting::CollectingCommands,
    },
    Cleaning {
        #[command(subcommand)]
        command: CleaningCommands,
    },
}

pub mod cleaning;
pub mod collecting;
pub mod region;
pub mod regional_taxa;
pub mod taxa;
