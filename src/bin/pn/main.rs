use std::collections::HashMap;

use anyhow::anyhow;
use clap::Parser;
use propagation_notebook::{
    collecting::{
        CleaningProcedure, CleaningProcedureStep, CollectingData, TaxonCleaningProcedure,
    },
    region::{Region, RegionalTaxonStatus},
    taxonomy::{Synonym, Taxon, VernacularName},
};
use tabled::settings::{Alignment, Modify, object::Columns};
use toasty::Db;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{
    MainCommand, Options, cleaning::CleaningCommands, collecting::CollectingCommands,
    region::RegionCommands, taxa::TaxonCommands,
};

mod cli;

fn truncate_with_summary(s: &str, max_chars: usize) -> String {
    let extra_chars = s.chars().count().saturating_sub(max_chars);
    if extra_chars == 0 {
        return s.to_string();
    }
    s.chars().take(max_chars).collect::<String>() + &format!("... [{extra_chars} more characters]")
}

fn join_or_default<T, F>(items: &[T], default: &str, extract: F) -> String
where
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        default.to_string()
    } else {
        items.iter().map(extract).collect::<Vec<_>>().join("\n")
    }
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
        MainCommand::Taxa { command } => match command {
            TaxonCommands::Search { search_string } => {
                tracing::debug!("Searching for exact complete name");
                if let Ok(found) = Taxon::filter(Taxon::fields().complete_name().eq(&search_string))
                    .one()
                    .exec(&mut db)
                    .await
                {
                    println!("found taxon {}", found.reference());
                } else {
                    tracing::debug!("Searching for approximate complete name");
                    let wildcard = format!("%{search_string}%");
                    let taxa = Taxon::filter(Taxon::fields().complete_name().like(&wildcard))
                        .exec(&mut db)
                        .await?;
                    if !taxa.is_empty() {
                        println!("Possible options for '{search_string}':");
                        for t in taxa {
                            println!("- {}", t.reference());
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
                                "Found '{}' which is a synonym for {}",
                                found.complete_name,
                                found.taxon.get().reference(),
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
                                        "'{}' is a synonym for {}",
                                        syn.complete_name,
                                        syn.taxon.get().reference(),
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
                                        "Found {} ({})",
                                        vernacular.taxon.get().reference(),
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
                                                "{} ({})",
                                                vernacular.taxon.get().reference(),
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
            TaxonCommands::Show { id } => {
                let taxon = Taxon::filter_by_id(id)
                    .include(Taxon::fields().parent())
                    .include(Taxon::fields().children())
                    .include(Taxon::fields().vernaculars())
                    .include(Taxon::fields().synonyms())
                    .include(Taxon::fields().regional_statuses().region())
                    .include(Taxon::fields().collecting_data())
                    .include(Taxon::fields().cleaning_procedure().procedure().steps())
                    .one()
                    .exec(&mut db)
                    .await?;
                {
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
                            .map(|p| format!("{} ({})", p.reference(), p.rank))
                            .unwrap_or_else(|| "-".into()),
                    ]);
                    tbuilder.push_record([
                        "Synonyms",
                        &join_or_default(taxon.synonyms.get(), "-", |v| v.complete_name.clone()),
                    ]);
                    tbuilder.push_record([
                        "Common Name(s)",
                        &join_or_default(taxon.vernaculars.get(), "-", |v| v.name.clone()),
                    ]);
                    tbuilder.push_record([
                        "Child taxa",
                        &join_or_default(taxon.children.get(), "-", |t| {
                            format!("{} ({})", t.reference(), t.rank)
                        }),
                    ]);
                    tbuilder.push_record([
                        "Regions",
                        &join_or_default(taxon.regional_statuses.get(), "-", |s| {
                            format!(
                                "{} ({})",
                                s.region.get().reference(),
                                s.origin
                                    .unwrap_or(propagation_notebook::region::Origin::Unknown)
                            )
                        }),
                    ]);
                    tbuilder.push_record([
                        "Ripening",
                        taxon
                            .collecting_data
                            .get()
                            .as_ref()
                            .map(|d| d.ripening_indicators.as_str())
                            .unwrap_or_else(|| "-"),
                    ]);
                    tbuilder.push_record([
                        "Storage",
                        taxon
                            .collecting_data
                            .get()
                            .as_ref()
                            .and_then(|d| d.storage.as_deref())
                            .unwrap_or("-"),
                    ]);
                    if let Some(tcp) = taxon.cleaning_procedure.get() {
                        tbuilder.push_record(["Seed Cleaning", &{
                            let proc = tcp.procedure.get();
                            let mut steps = Vec::from(proc.steps.get());
                            steps.sort_by_key(|v| v.order);
                            let mut inner_table = tabled::builder::Builder::default();
                            inner_table.push_record(["ID", &proc.id.to_string()]);
                            inner_table.push_record(["Name", &proc.name]);
                            inner_table
                                .push_record(["Notes", proc.notes.as_deref().unwrap_or("-")]);
                            inner_table.push_record([
                                "Steps",
                                &join_or_default(&steps, "[none]", |step| {
                                    format!(" - {}", step.summary())
                                }),
                            ]);
                            let s = format!(
                                "Taxon-specific Notes:\n{}\n\nProcedure:\n{}",
                                tcp.notes.as_deref().unwrap_or("[none]"),
                                inner_table.build().with(tabled::settings::Style::blank())
                            );
                            s
                        }])
                    }
                    println!(
                        "{}",
                        tbuilder
                            .build()
                            .with(tabled::settings::Style::blank())
                            .with(Modify::new(Columns::first()).with(Alignment::right()))
                    );
                    println!();
                    println!("Regional Information:");
                    for status in taxon.regional_statuses.get() {
                        let mut tbuilder = tabled::builder::Builder::default();
                        tbuilder.push_record(["Region", &status.region.get().reference()]);
                        tbuilder.push_record([
                            "Origin",
                            &status
                                .origin
                                .unwrap_or(propagation_notebook::region::Origin::Unknown)
                                .to_string(),
                        ]);
                        tbuilder.push_record([
                            "C-value",
                            &status
                                .c_value
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "-".into()),
                        ]);
                        tbuilder.push_record([
                            "Conservation Status",
                            &status
                                .conservation_status
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "-".into()),
                        ]);
                        tbuilder.push_record([
                            "Wetland Indicator",
                            &status
                                .wetland_indicator
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "-".into()),
                        ]);
                        let window_str = match (status.window_start, status.window_end) {
                            (None, None) => "-".into(),
                            _ => format!(
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
                        tbuilder.push_record(["Harvest Window", &window_str]);
                        println!(
                            "{}",
                            tbuilder
                                .build()
                                .with(tabled::settings::Style::blank())
                                .with(Modify::new(Columns::first()).with(Alignment::right()))
                        );
                        println!();
                    }
                }
            }
            TaxonCommands::List => {
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
            TaxonCommands::SetCleaningProcedure {
                taxon_id,
                procedure_id,
                notes,
                remove,
            } => {
                if remove {
                    if inquire::Confirm::new("Are you sure you wish to remove this procedure?")
                        .with_default(false)
                        .prompt()?
                    {
                        TaxonCleaningProcedure::delete_by_taxon_id_and_procedure_id(
                            &mut db,
                            taxon_id,
                            procedure_id,
                        )
                        .await?;
                        println!("Assignment removed");
                    }
                } else {
                    TaxonCleaningProcedure::create()
                        .taxon_id(taxon_id)
                        .procedure_id(procedure_id)
                        .notes(notes)
                        .exec(&mut db)
                        .await?;
                    println!("Procedure {} assigned to taxon {}", taxon_id, procedure_id);
                }
            }
        },
        MainCommand::Regions { command } => match command {
            RegionCommands::List => {
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
            RegionCommands::Show { id } => {
                let region = Region::filter_by_id(id)
                    .include(Region::fields().taxon_statuses())
                    .one()
                    .exec(&mut db)
                    .await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &region.id.to_string()]);
                tbuilder.push_record(["Name", &region.name]);
                tbuilder.push_record(["Notes", &region.notes.unwrap_or_else(|| "-".to_string())]);
                tbuilder.push_record(["Taxa", &region.taxon_statuses.get().len().to_string()]);
                tbuilder.push_record([
                    "Bounds",
                    &truncate_with_summary(&region.bounds.unwrap_or_else(|| "-".to_string()), 500),
                ]);
                println!(
                    "{}",
                    tbuilder
                        .build()
                        .with(tabled::settings::Style::blank())
                        .with(Modify::new(Columns::first()).with(Alignment::right()))
                )
            }
            RegionCommands::Modify {
                id,
                bounds,
                name,
                notes,
            } => {
                let mut update_query = Region::update_by_id(id);
                let bounds = bounds.resolve().await?;
                if let Some(name) = name {
                    update_query = update_query.name(name);
                }
                if let Some(bounds) = bounds {
                    update_query = update_query.bounds(bounds);
                }
                if let Some(notes) = notes {
                    update_query = update_query.notes(notes);
                }
                update_query.exec(&mut db).await?;
                println!("Region {id} updated");
            }
            RegionCommands::Add {
                region_name,
                bounds,
                notes,
            } => {
                let bounds = bounds.resolve().await?;
                let new_region = Region::create()
                    .name(region_name)
                    .bounds(bounds)
                    .notes(notes)
                    .exec(&mut db)
                    .await?;
                println!("Added new region {}", new_region.reference());
            }
            RegionCommands::Remove { id } => {
                if inquire::Confirm::new("Are you sure you wish to delete this region?")
                    .with_default(false)
                    .with_help_message("All associated data will be deleted")
                    .prompt()?
                {
                    Region::delete_by_id(&mut db, id).await?;
                    println!("Deleted region {id} from the database");
                }
            }
            RegionCommands::AddTaxon { id, props } => {
                let s = RegionalTaxonStatus::create()
                    .region_id(id.region_id)
                    .taxon_id(id.taxon_id)
                    .origin(props.origin)
                    .c_value(props.c_value)
                    .conservation_status(props.conservation_status)
                    .wetland_indicator(props.wetland_indicator)
                    .window_start(
                        props
                            .harvest_start
                            .map(|d| d.with().year(2000).build().unwrap()),
                    )
                    .window_end(
                        props
                            .harvest_end
                            .map(|d| d.with().year(2000).build().unwrap()),
                    )
                    .exec(&mut db)
                    .await?;
                println!("Added regional taxon {}", s.id);
            }
            RegionCommands::ModifyTaxon { id, props } => {
                let mut query = RegionalTaxonStatus::update_by_taxon_id_and_region_id(
                    id.taxon_id,
                    id.region_id,
                );
                if let Some(origin) = props.origin {
                    query = query.origin(origin);
                }
                if let Some(c_value) = props.c_value {
                    query = query.c_value(c_value);
                }
                if let Some(conservation_status) = props.conservation_status {
                    query = query.conservation_status(conservation_status);
                }
                if let Some(wetland_indicator) = props.wetland_indicator {
                    query = query.wetland_indicator(wetland_indicator);
                }
                if let Some(harvest_start) = props.harvest_start {
                    query = query.window_start(harvest_start.with().year(2000).build().unwrap());
                }
                if let Some(harvest_end) = props.harvest_end {
                    query = query.window_end(harvest_end.with().year(2000).build().unwrap());
                }
                query.exec(&mut db).await?;
                println!("Modified taxon {} in region {}", id.taxon_id, id.region_id);
            }
            RegionCommands::ListTaxa { region_id } => {
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

                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Taxon", "Origin"]);
                for taxon in taxa {
                    let status = map.get(&taxon.id).unwrap();
                    tbuilder.push_record([
                        taxon.id.to_string(),
                        taxon.complete_name,
                        status
                            .origin
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "-".into()),
                    ]);
                }
                println!(
                    "{}",
                    tbuilder.build().with(tabled::settings::Style::blank())
                );
            }
            RegionCommands::RemoveTaxon { id } => {
                if inquire::Confirm::new("Are you sure you wish to remove this regional taxon?")
                    .with_default(false)
                    .prompt()?
                {
                    RegionalTaxonStatus::delete_by_taxon_id_and_region_id(
                        &mut db,
                        id.taxon_id,
                        id.region_id,
                    )
                    .await?;
                    println!("Removed taxon {} from region {}", id.taxon_id, id.region_id);
                }
            }
        },
        MainCommand::Collecting { command } => match command {
            CollectingCommands::List => {
                let items = CollectingData::all()
                    .include(CollectingData::fields().taxon())
                    .exec(&mut db)
                    .await?;
                let nitems = items.len();
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Taxon"]);
                for item in items {
                    tbuilder.push_record([item.id.to_string(), item.taxon.get().reference()])
                }
                println!(
                    "{}",
                    tbuilder.build().with(tabled::settings::Style::blank())
                );
                println!("\n{nitems} found");
            }
            CollectingCommands::Show { id, taxon_id } => {
                let data = match (id, taxon_id) {
                    (Some(id), None) => CollectingData::filter_by_id(id),
                    (None, Some(taxon_id)) => CollectingData::filter_by_taxon_id(taxon_id),
                    _ => return Err(anyhow!("must specify either an id or a taxon id")),
                }
                .include(CollectingData::fields().taxon())
                .one()
                .exec(&mut db)
                .await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &data.id.to_string()]);
                tbuilder.push_record(["Taxon", &data.taxon.get().reference()]);
                tbuilder.push_record(["Ripening", &data.ripening_indicators]);
                tbuilder.push_record(["Storage", &data.storage.unwrap_or_else(|| "-".into())]);
                println!(
                    "{}",
                    tbuilder
                        .build()
                        .with(tabled::settings::Style::blank())
                        .with(Modify::new(Columns::first()).with(Alignment::right()))
                )
            }
            CollectingCommands::Add {
                taxon_id,
                ripening_indicators,
                storage,
            } => {
                let data = CollectingData::create()
                    .taxon_id(taxon_id)
                    .ripening_indicators(ripening_indicators)
                    .storage(storage)
                    .exec(&mut db)
                    .await?;
                println!("Added collection information {}", data.id);
            }
            CollectingCommands::Remove { id } => {
                if inquire::Confirm::new("Are you sure you wish to remove this collecting data?")
                    .with_default(false)
                    .prompt()?
                {
                    CollectingData::delete_by_id(&mut db, id).await?;
                    println!("Removed collecting data {id}")
                }
            }
            CollectingCommands::Modify {
                id,
                ripening_indicators,
                storage,
            } => {
                let mut query = CollectingData::update_by_id(id);
                if let Some(ripening) = ripening_indicators {
                    query = query.ripening_indicators(ripening);
                }
                if let Some(storage) = storage {
                    query = query.storage(storage);
                }
                query.exec(&mut db).await?;
                println!("Modified collection information {id}");
            }
        },
        MainCommand::Cleaning { command } => match command {
            CleaningCommands::List => {
                let items = CleaningProcedure::all()
                    .include(CleaningProcedure::fields().steps())
                    .include(CleaningProcedure::fields().taxon_links().taxon())
                    .exec(&mut db)
                    .await?;
                let nitems = items.len();
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Name", "Steps", "Taxa"]);
                for item in items {
                    tbuilder.push_record([
                        item.id.to_string(),
                        item.name,
                        item.steps.get().len().to_string(),
                        item.taxon_links.get().len().to_string(),
                    ])
                }
                println!(
                    "{}",
                    tbuilder.build().with(tabled::settings::Style::blank())
                );
                println!("\n{nitems} found");
            }
            CleaningCommands::Show { id } => {
                let procedure = CleaningProcedure::filter_by_id(id)
                    .include(CleaningProcedure::fields().steps())
                    .include(CleaningProcedure::fields().taxon_links().taxon())
                    .one()
                    .exec(&mut db)
                    .await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", &procedure.id.to_string()]);
                tbuilder.push_record(["Name", &procedure.name]);
                tbuilder.push_record(["Notes", &procedure.notes.unwrap_or_else(|| "-".into())]);
                tbuilder.push_record([
                    "Taxa",
                    &join_or_default(procedure.taxon_links.get(), "-", |v| {
                        v.taxon.get().reference()
                    }),
                ]);
                // sort steps in order
                let mut steps = Vec::from(procedure.steps.get());
                steps.sort_by_key(|a| a.order);
                tbuilder.push_record([
                    "Steps",
                    &join_or_default(&steps, "-", |step| format!(" - {}", step.summary())),
                ]);
                println!(
                    "{}",
                    tbuilder.build().with(tabled::settings::Style::blank())
                );
            }
            CleaningCommands::Add { name, notes } => {
                let item = CleaningProcedure::create()
                    .name(name)
                    .notes(notes)
                    .exec(&mut db)
                    .await?;
                println!("Added new procedure {}", item.id);
            }
            CleaningCommands::AddStep {
                procedure_id,
                order,
                step_type,
                equipment,
                notes,
            } => {
                let step = CleaningProcedureStep::create()
                    .procedure_id(procedure_id)
                    .order(order)
                    .operation_type(step_type)
                    .equipment(equipment)
                    .notes(notes)
                    .exec(&mut db)
                    .await?;
                println!("Added new step {}", step.id);
            }
            CleaningCommands::Steps { procedure_id } => {
                let steps = CleaningProcedureStep::filter_by_procedure_id(procedure_id)
                    .order_by(CleaningProcedureStep::fields().order().asc())
                    .exec(&mut db)
                    .await?;
                let mut table = tabled::Table::new(steps.iter());

                println!("{}", table.with(tabled::settings::Style::blank()));
            }
            CleaningCommands::ModifyStep {
                id,
                order,
                step_type,
                equipment,
                notes,
            } => {
                let mut query = CleaningProcedureStep::update_by_id(id);
                if let Some(order) = order {
                    query = query.order(order);
                }
                if let Some(step_type) = step_type {
                    query = query.operation_type(step_type);
                }
                if let Some(equipment) = equipment {
                    query = query.equipment(equipment);
                }
                if let Some(notes) = notes {
                    query = query.notes(notes);
                }
                query.exec(&mut db).await?;
                println!("Updated step {}", id);
            }
            CleaningCommands::Remove { id } => {
                if inquire::Confirm::new("Are you sure you wish to remove this cleaning procedure?")
                    .with_default(false)
                    .with_help_message("It will remove all related steps")
                    .prompt()?
                {
                    CleaningProcedure::delete_by_id(&mut db, id).await?;
                    println!("Removed cleaning procedure {id}");
                }
            }
            CleaningCommands::Modify { id, name, notes } => {
                let mut query = CleaningProcedure::update_by_id(id);
                if let Some(name) = name {
                    query = query.name(name);
                }
                if let Some(notes) = notes {
                    query = query.notes(notes);
                }
                query.exec(&mut db).await?;
                println!("Modified cleaning procedure {id}");
            }
            CleaningCommands::RemoveStep { id } => {
                if inquire::Confirm::new("Are you sure you wish to remove this step?")
                    .with_default(false)
                    .prompt()?
                {
                    CleaningProcedureStep::delete_by_id(&mut db, id).await?;
                    println!("Removed step {id}");
                }
            }
        },
    };
    Ok(())
}
