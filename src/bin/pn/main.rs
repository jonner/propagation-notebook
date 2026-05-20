use std::collections::HashMap;

use clap::Parser;
use propagation_notebook::{
    region::{Region, RegionalTaxonStatus},
    taxonomy::{Synonym, Taxon, VernacularName},
};
use tabled::settings::{Alignment, Modify, object::Columns};
use toasty::Db;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::Options;

mod cli;

fn truncate_with_summary(s: &str, max_chars: usize) -> String {
    let extra_chars = s.chars().count().saturating_sub(max_chars);
    if extra_chars == 0 {
        return s.to_string();
    }
    s.chars().take(max_chars).collect::<String>() + &format!("... [{extra_chars} more characters]")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::default().add_directive(LevelFilter::WARN.into()));
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();
    let options = Options::parse();
    let mut db = Db::builder()
        .models(propagation_notebook::models())
        .connect("sqlite:./propagation-notebook.sqlite")
        .await?;
    match options.command {
        cli::MainCommand::Taxa { command } => match command {
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
            cli::TaxonCommands::Show { id } => {
                let taxon = Taxon::filter_by_id(id)
                    .include(Taxon::fields().parent())
                    .include(Taxon::fields().children())
                    .include(Taxon::fields().vernaculars())
                    .include(Taxon::fields().synonyms())
                    .include(Taxon::fields().regional_statuses().region())
                    .one()
                    .exec(&mut db)
                    .await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &taxon.id.to_string()]);
                tbuilder.push_record(["Name", &taxon.complete_name]);
                tbuilder.push_record(["Rank", &taxon.rank.to_string()]);
                tbuilder.push_record([
                    "Parent",
                    &taxon
                        .parent
                        .get()
                        .as_ref()
                        .map(|p| format!("{}: {} ({})", p.id, p.complete_name, p.rank))
                        .unwrap_or_default(),
                ]);
                tbuilder.push_record([
                    "Synonyms",
                    &taxon
                        .synonyms
                        .get()
                        .iter()
                        .map(|s| s.complete_name.clone())
                        .collect::<Vec<_>>()
                        .join("\n"),
                ]);
                tbuilder.push_record([
                    "Common Name(s)",
                    &taxon
                        .vernaculars
                        .get()
                        .iter()
                        .map(|v| v.name.clone())
                        .collect::<Vec<_>>()
                        .join("\n"),
                ]);
                tbuilder.push_record([
                    "Child taxa",
                    &taxon
                        .children
                        .get()
                        .iter()
                        .map(|t| format!("{}: {} ({})", t.id, t.complete_name, t.rank))
                        .collect::<Vec<_>>()
                        .join("\n"),
                ]);
                tbuilder.push_record([
                    "Regions",
                    &taxon
                        .regional_statuses
                        .get()
                        .iter()
                        .map(|s| {
                            format!(
                                "{}: {} ({})",
                                s.region_id,
                                s.region.get().name,
                                s.native_status
                                    .unwrap_or(propagation_notebook::region::NativeStatus::Unknown)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                ]);
                println!(
                    "{}",
                    tbuilder
                        .build()
                        .with(tabled::settings::Style::blank())
                        .with(Modify::new(Columns::first()).with(Alignment::right()))
                );
            }
            cli::TaxonCommands::List => {
                let taxa = Taxon::all()
                    .order_by(Taxon::fields().sequence().asc())
                    .exec(&mut db)
                    .await?;
                let ntaxa = taxa.len();

                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Name"]);
                for taxon in taxa {
                    tbuilder.push_record([taxon.id.to_string(), taxon.complete_name]);
                }
                println!(
                    "{}",
                    tbuilder.build().with(tabled::settings::Style::blank())
                );
                println!("{} taxa found", ntaxa);
            }
        },
        cli::MainCommand::Regions { command } => match command {
            cli::RegionCommands::List => {
                let regions = Region::all().exec(&mut db).await?;
                if regions.is_empty() {
                    println!("No Regions found");
                } else {
                    let mut tbuilder = tabled::builder::Builder::default();
                    tbuilder.push_record(["ID", "Name"]);
                    for region in regions {
                        tbuilder.push_record([region.id.to_string(), region.name])
                    }
                    println!(
                        "{}",
                        tbuilder.build().with(tabled::settings::Style::blank())
                    );
                }
            }
            cli::RegionCommands::Show { id } => {
                let region = Region::get_by_id(&mut db, id).await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &region.id.to_string()]);
                tbuilder.push_record(["Name", &region.name]);
                tbuilder.push_record([
                    "Bounds",
                    &truncate_with_summary(&region.bounds.unwrap_or("None".to_string()), 500),
                ]);
                println!(
                    "{}",
                    tbuilder
                        .build()
                        .with(tabled::settings::Style::blank())
                        .with(Modify::new(Columns::first()).with(Alignment::right()))
                )
            }
            cli::RegionCommands::Modify { id, bounds, name } => {
                let mut update_query = Region::update_by_id(id);
                let bounds = bounds.resolve().await?;
                if let Some(name) = name {
                    update_query = update_query.name(name);
                }
                if let Some(bounds) = bounds {
                    update_query = update_query.bounds(bounds);
                }
                update_query.exec(&mut db).await?;
                println!("Region {id} updated");
            }
            cli::RegionCommands::Add {
                region_name,
                bounds,
            } => {
                let bounds = bounds.resolve().await?;
                let new_region = Region::create()
                    .name(region_name)
                    .bounds(bounds)
                    .exec(&mut db)
                    .await?;
                println!("Added new region {}: {}", new_region.id, new_region.name);
            }
        },
        cli::MainCommand::RegionalTaxa { command } => match command {
            cli::RegionalTaxaCommands::Add {
                region_id,
                taxon_id,
                native_status,
                c_value,
                conservation_status,
                wetland_indicator,
                harvest_start,
                harvest_end,
            } => {
                let s = RegionalTaxonStatus::create()
                    .region_id(region_id)
                    .taxon_id(taxon_id)
                    .native_status(native_status)
                    .c_value(c_value)
                    .conservation_status(conservation_status)
                    .wetland_indicator(wetland_indicator)
                    .window_start(harvest_start.map(|d| d.with().year(2000).build().unwrap()))
                    .window_end(harvest_end.map(|d| d.with().year(2000).build().unwrap()))
                    .exec(&mut db)
                    .await?;
                println!("Added regional taxon {}", s.id);
            }
            cli::RegionalTaxaCommands::List { region_id } => {
                let region = Region::get_by_id(&mut db, region_id).await?;
                let regional_statuses = RegionalTaxonStatus::filter(
                    RegionalTaxonStatus::fields().region_id().eq(region_id),
                )
                // FIXME: We want to order by a taxon sequence, but
                // toasty doesn't yet support ordering by data in a relation
                .exec(&mut db)
                .await?;

                // FIXME: it's too slow to include all relations, so query the taxa separately
                let taxa = Taxon::filter(
                    Taxon::fields().id().in_list(
                        regional_statuses
                            .iter()
                            .map(|s| s.taxon_id)
                            .collect::<Vec<_>>(),
                    ),
                )
                .order_by(Taxon::fields().sequence().asc())
                .exec(&mut db)
                .await?;

                // since we can't order the regional status list by taxon
                // sequence, we need to iterate through the sorted taxon list, and then look up the
                // regional status from a hash table
                let map = regional_statuses
                    .into_iter()
                    .map(|s| (s.taxon_id, s))
                    .collect::<HashMap<_, _>>();

                let ntaxa = taxa.len();
                println!("Regional Taxa from region '{}'", region.name);
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Taxon ID", "Name", "Origin"]);
                for taxon in taxa {
                    let status = map.get(&taxon.id).unwrap();
                    tbuilder.push_record([
                        status.id.to_string(),
                        taxon.id.to_string(),
                        taxon.complete_name,
                        status
                            .native_status
                            .map(|s| s.to_string())
                            .unwrap_or_default(),
                    ]);
                }
                println!(
                    "{}",
                    tbuilder.build().with(tabled::settings::Style::blank())
                );
                println!("{} taxa found", ntaxa);
            }
            cli::RegionalTaxaCommands::Show { id } => {
                let status = RegionalTaxonStatus::filter_by_id(id)
                    .include(RegionalTaxonStatus::fields().region())
                    .include(RegionalTaxonStatus::fields().taxon())
                    .one()
                    .exec(&mut db)
                    .await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &status.id.to_string()]);
                tbuilder.push_record([
                    "Taxon",
                    &format!(
                        "{}: {}",
                        status.taxon.get().id,
                        status.taxon.get().complete_name
                    ),
                ]);
                tbuilder.push_record([
                    "Region",
                    &format!("{}: {}", status.region.get().id, status.region.get().name),
                ]);
                tbuilder.push_record([
                    "Origin",
                    &status
                        .native_status
                        .unwrap_or(propagation_notebook::region::NativeStatus::Unknown)
                        .to_string(),
                ]);
                tbuilder.push_record([
                    "Coeff. of conservatism",
                    &status.c_value.map(|v| v.to_string()).unwrap_or_default(),
                ]);
                tbuilder.push_record([
                    "Conservation Status",
                    &status
                        .conservation_status
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ]);
                tbuilder.push_record([
                    "Wetland Indicator",
                    &status
                        .wetland_indicator
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ]);
                let window_str = match (status.window_start, status.window_end) {
                    (None, None) => "",
                    _ => &format!(
                        "{} - {}",
                        status
                            .window_start
                            .map(|d| d.strftime("%b %d").to_string())
                            .unwrap_or("?".to_string()),
                        status
                            .window_end
                            .map(|d| d.strftime("%b %d").to_string())
                            .unwrap_or("?".to_string())
                    ),
                };
                tbuilder.push_record(["Harvest Window", window_str]);
                // pub window_start: Option<jiff::civil::Date>,
                // pub window_end: Option<jiff::civil::Date>,
                println!(
                    "{}",
                    tbuilder
                        .build()
                        .with(tabled::settings::Style::blank())
                        .with(Modify::new(Columns::first()).with(Alignment::right()))
                )
            }
            cli::RegionalTaxaCommands::Remove { id } => {
                RegionalTaxonStatus::delete_by_id(&mut db, id).await?;
                println!("Deleted regional taxon {id}");
            }
        },
    }
    Ok(())
}
