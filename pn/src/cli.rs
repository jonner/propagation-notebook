use std::collections::HashMap;

use libpropagation::{region::RegionalTaxonStatus, taxonomy::Taxon};

use crate::{
    cli::{cleaning::CleaningCommands, propagation::PropagationCommands},
    style,
};

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

// sorts by taxa sequence and then prints a table. Expects `regional_statuses`
// to have unloaded taxon fields
async fn print_regional_taxa_table(
    db: &mut toasty::Db,
    regional_statuses: Vec<RegionalTaxonStatus>,
) -> Result<(), anyhow::Error> {
    let map = regional_statuses
        .into_iter()
        .map(|s| (s.taxon_id, s))
        .collect::<HashMap<_, _>>();

    let taxa = Taxon::filter(Taxon::fields().id().in_list(map.keys().collect::<Vec<_>>()))
        .order_by(Taxon::fields().sequence().asc())
        .exec(db)
        .await?;

    let mut tbuilder = tabled::builder::Builder::default();
    tbuilder.push_record([
        "ID",
        "Taxon",
        "Origin",
        "Status",
        "C-value",
        "Wetland Indicator",
    ]);
    // since we can't order the regional status list by taxon sequence, we need
    // to iterate through the sorted taxon list, and then look up the regional
    // status from a hash table
    for taxon in taxa {
        let status = map.get(&taxon.id).unwrap();
        tbuilder.push_record([
            taxon.id.to_string(),
            taxon.complete_name,
            status
                .origin
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
            status
                .conservation_status
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
            status
                .c_value
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
            status
                .wetland_indicator
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
        ]);
    }
    println!("{}", tbuilder.build().with(style::ListTable));
    Ok(())
}
