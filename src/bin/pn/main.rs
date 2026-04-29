use clap::Parser;
use propagation_notebook::{
    region::Region,
    taxonomy::{Synonym, Taxon, VernacularName},
};
use toasty::Db;

use crate::cli::Options;

mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let options = Options::parse();
    let mut db = Db::builder()
        .models(propagation_notebook::models())
        .connect("sqlite:./propagation-notebook.sqlite")
        .await?;
    match options.command {
        cli::MainCommand::Taxon { command } => match command {
            cli::TaxonCommands::Search { search_string } => {
                tracing::debug!("Searching for exact complete name");
                if let Ok(found) = Taxon::filter(Taxon::fields().complete_name().eq(&search_string))
                    .one()
                    .exec(&mut db)
                    .await
                {
                    println!("found taxon {}: '{}'", found.id, found.complete_name);
                } else {
                    tracing::debug!("Searching for approximate complete name");
                    let wildcard = format!("%{search_string}%");
                    let taxa = Taxon::filter(Taxon::fields().complete_name().like(&wildcard))
                        .exec(&mut db)
                        .await?;
                    if !taxa.is_empty() {
                        println!("Possible options for '{search_string}':");
                        for t in taxa {
                            println!("- {}: {}", t.id, t.complete_name);
                        }
                    } else {
                        tracing::debug!("Searching for exact scientific synonym");
                        if let Ok(found) =
                            Synonym::filter(Synonym::fields().complete_name().eq(&search_string))
                                .include(Synonym::fields().taxon())
                                .one()
                                .exec(&mut db)
                                .await
                        {
                            println!(
                                "Found '{}' which is a synonym for {}: '{}'",
                                found.complete_name,
                                found.taxon.get().id,
                                found.taxon.get().complete_name
                            );
                        } else {
                            tracing::debug!("Searching for approximate scientific synonyms");
                            let synonyms =
                                Synonym::filter(Synonym::fields().complete_name().like(&wildcard))
                                    .include(Synonym::fields().taxon())
                                    .exec(&mut db)
                                    .await?;
                            if !synonyms.is_empty() {
                                println!("Possible options for '{search_string}':");
                                for syn in synonyms {
                                    println!(
                                        "'{}' is a synonym for {}: '{}'",
                                        syn.complete_name,
                                        syn.taxon.get().id,
                                        syn.taxon.get().complete_name
                                    );
                                }
                            } else {
                                tracing::debug!("Searching for exact vernacular name");
                                // look up common names
                                if let Ok(vernacular) = VernacularName::filter(
                                    VernacularName::fields().name().eq(&search_string),
                                )
                                .include(VernacularName::fields().taxon())
                                .one()
                                .exec(&mut db)
                                .await
                                {
                                    println!(
                                        "Found {}: '{} ({})'",
                                        vernacular.taxon.get().id,
                                        vernacular.taxon.get().complete_name,
                                        vernacular.name
                                    );
                                } else {
                                    tracing::debug!("Searching for approximate vernacular names");
                                    let vernaculars = VernacularName::filter(
                                        VernacularName::fields().name().like(&wildcard),
                                    )
                                    .include(VernacularName::fields().taxon())
                                    .exec(&mut db)
                                    .await?;
                                    if !vernaculars.is_empty() {
                                        println!("Possible options for '{search_string}':");
                                        for vernacular in vernaculars {
                                            println!(
                                                "{}: '{} ({})'",
                                                vernacular.taxon.get().id,
                                                vernacular.taxon.get().complete_name,
                                                vernacular.name,
                                            );
                                        }
                                    } else {
                                        println!("No options found");
                                    }
                                }
                            }
                        }
                    }
                }
            }
            cli::TaxonCommands::Info { id } => {
                let taxon = Taxon::filter_by_id(id)
                    .include(Taxon::fields().parent())
                    .include(Taxon::fields().children())
                    .include(Taxon::fields().vernaculars())
                    .include(Taxon::fields().synonyms())
                    .one()
                    .exec(&mut db)
                    .await?;
                // dbg!(&taxon);
                println!("{}", taxon.complete_name);
                println!("{}", "=".repeat(taxon.complete_name.len()));
                println!("ID: {}", taxon.id);
                println!("Rank: {}", taxon.rank);
                if let Some(parent) = taxon.parent.get() {
                    println!("Parent: {} ({})", parent.complete_name, parent.rank)
                }
                if !taxon.synonyms.get().is_empty() {
                    println!("Synonym(s):");
                    for syn in taxon.synonyms.get() {
                        println!(" - {}", syn.complete_name);
                    }
                }
                if !taxon.vernaculars.get().is_empty() {
                    println!("Common Name(s):");
                    for vernacular in taxon.vernaculars.get() {
                        println!(" - {}", vernacular.name);
                    }
                }
                if !taxon.children.get().is_empty() {
                    println!("Child taxa:");
                    for child in taxon.children.get() {
                        println!(" - {}: {} ({})", child.id, child.complete_name, child.rank);
                    }
                }
            }
        },
        cli::MainCommand::Region { command } => match command {
            cli::RegionCommands::List => {
                let regions = Region::all().exec(&mut db).await?;
                println!("{} regions found", regions.len());
                for region in regions {
                    dbg!(&region);
                }
            }
            cli::RegionCommands::Add { region_name } => {
                let new_region = Region::create().name(region_name).exec(&mut db).await?;
                println!("Added new region {}: {}", new_region.id, new_region.name);
            }
        },
    }
    Ok(())
}
