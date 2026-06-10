use std::path::PathBuf;

use propagation_notebook::region::{
    ConservationStatus, Origin, Region, RegionalHarvestWindow, RegionalTaxonStatus,
    WetlandIndicator,
};
use toasty::Db;

use crate::{cli::list_regional_taxa, style, util::truncate_with_summary};

mod import;

#[derive(clap::Args, Debug)]
#[group(required = false, multiple = false)]
pub struct BoundsArg {
    #[arg(
        long,
        help = "path to a geojson file whose contents represent the bounds of the region",
        conflicts_with = "bounds_string"
    )]
    pub bounds_file: Option<PathBuf>,
    #[arg(
        short,
        long = "bounds",
        help = "geojson string representing the bounds of the region",
        conflicts_with = "bounds_file"
    )]
    pub bounds_string: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct RegionalTaxonProperties {
    #[arg(short, long, help = "Origin of the taxon vis-a-vis this region")]
    pub origin: Option<Origin>,
    #[arg(
        long,
        help = "Coefficient of conservatism (0-10) for the species in this region"
    )]
    pub c_value: Option<u64>,
    #[arg(
        short,
        long,
        help = "Conservation status for the species in the given region"
    )]
    pub conservation_status: Option<ConservationStatus>,
    #[arg(
        short,
        long,
        help = "Whether the species is a wetland indicator in the given region"
    )]
    pub wetland_indicator: Option<WetlandIndicator>,
    // harvest phenology
    #[arg(
        long,
        help = "Start of the harvest window for the species in the given region (format: MM-DD)",
        value_parser = parse_month_day,
    )]
    pub harvest_start: Option<jiff::civil::Date>,
    #[arg(
        long,
        help = "End of the harvest window for the species in the given region (format: MM-DD)",
        value_parser = parse_month_day,
    )]
    pub harvest_end: Option<jiff::civil::Date>,
}

pub fn parse_month_day(input: &str) -> anyhow::Result<jiff::civil::Date> {
    let mut parsed = jiff::fmt::strtime::parse("%m-%d", input)?;
    parsed.set_year(Some(2000))?;
    parsed.to_date().map_err(|e| e.into())
}

impl BoundsArg {
    pub async fn resolve(&self) -> anyhow::Result<Option<String>> {
        match (self.bounds_string.as_ref(), self.bounds_file.as_ref()) {
            (Some(bounds_string), None) => Ok(Some(bounds_string.clone())),
            (None, Some(bounds_file)) => Ok(Some(tokio::fs::read_to_string(bounds_file).await?)),
            (None, None) => Ok(None),
            _ => Err(anyhow::anyhow!(
                "Only one of 'bounds' or 'bounds_file' can be specified at the same time"
            )),
        }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum RegionCommands {
    #[command(about = "Print a list of regions")]
    List,
    #[command(about = "Show detailed information about a region")]
    Show { id: u64 },
    #[command(about = "Add a new region to the database")]
    Add {
        region_name: String,
        #[clap(flatten)]
        bounds: BoundsArg,
        #[arg(long, help = "Free-form notes about the region")]
        notes: Option<String>,
    },
    #[command(about = "Import a new region to the database")]
    Import {
        #[arg(help = "A path to a yaml file describing a region")]
        path: PathBuf,
    },
    #[command(about = "Modify information about a region", group(clap::ArgGroup::new("modify_fields").args(["name", "bounds_string", "bounds_file", "notes"]).required(true).multiple(true)))]
    Modify {
        id: u64,
        #[command(flatten)]
        bounds: BoundsArg,
        #[arg(short, long, help = "Specify a new name for the region")]
        name: Option<String>,
        #[arg(long, help = "Set notes for a region")]
        notes: Option<String>,
    },
    #[command(about = "Remove a region from the database")]
    Remove {
        id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    #[command(about = "Manage taxa for a region")]
    Taxa {
        #[arg(short, long, help = "ID of a region")]
        region_id: u64,
        #[command(subcommand)]
        command: RegionTaxaCommands,
    },
}

impl RegionCommands {
    pub async fn run(&self, db: &mut Db) -> anyhow::Result<()> {
        match self {
            RegionCommands::List => {
                let regions = Region::all()
                    .include(Region::fields().taxon_statuses())
                    .exec(db)
                    .await?;
                if regions.is_empty() {
                    println!("No Regions found");
                } else {
                    let mut tbuilder = tabled::builder::Builder::default();
                    tbuilder.push_record(["ID", "Name", "Taxa"]);
                    for region in regions {
                        tbuilder.push_record([
                            region.id.to_string(),
                            region.name,
                            region.taxon_statuses.get().len().to_string(),
                        ])
                    }
                    println!("{}", tbuilder.build().with(style::BasicTable));
                }
            }
            RegionCommands::Show { id } => {
                let region = Region::filter_by_id(id)
                    .include(Region::fields().taxon_statuses())
                    .one()
                    .exec(db)
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
                println!("{}", tbuilder.build().with(style::DetailTable))
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
                update_query.exec(db).await?;
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
                    .exec(db)
                    .await?;
                println!("Added new region {}", new_region.reference());
            }
            RegionCommands::Import { path } => {
                import::import_region(db, path).await?;
            }
            RegionCommands::Remove { id, assumeyes } => {
                if *assumeyes
                    || inquire::Confirm::new("Are you sure you wish to delete this region?")
                        .with_default(false)
                        .with_help_message("All associated data will be deleted")
                        .prompt()?
                {
                    Region::delete_by_id(db, id).await?;
                    println!("Deleted region {id} from the database");
                }
            }
            RegionCommands::Taxa { region_id, command } => match command {
                RegionTaxaCommands::Show { taxon_id } => {
                    let status =
                        RegionalTaxonStatus::filter_by_taxon_id_and_region_id(taxon_id, region_id)
                            .include(RegionalTaxonStatus::fields().region())
                            .include(RegionalTaxonStatus::fields().taxon())
                            .one()
                            .exec(db)
                            .await?;
                    let mut tbuilder = tabled::builder::Builder::default();
                    tbuilder.push_record(["Taxon", &status.taxon.get().reference()]);
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
                    tbuilder.push_record(["Harvest Window", &status.harvest_window.to_string()]);
                    println!("{}", tbuilder.build().with(style::DetailTable));
                    println!();
                }
                RegionTaxaCommands::Add { taxon_id, props } => {
                    // make sure region exists
                    let _r = Region::get_by_id(db, region_id).await?;
                    let s = RegionalTaxonStatus::create()
                        .region_id(region_id)
                        .taxon_id(taxon_id)
                        .origin(props.origin)
                        .c_value(props.c_value)
                        .conservation_status(props.conservation_status)
                        .wetland_indicator(props.wetland_indicator)
                        .harvest_window(RegionalHarvestWindow {
                            start: props
                                .harvest_start
                                .map(|d| d.with().year(2000).build().unwrap()),
                            end: props
                                .harvest_end
                                .map(|d| d.with().year(2000).build().unwrap()),
                        })
                        .exec(db)
                        .await?;
                    println!("Added regional taxon {}", s.id);
                }
                RegionTaxaCommands::Modify { taxon_id, props } => {
                    // make sure region exists
                    let _r = Region::get_by_id(db, region_id).await?;
                    let mut query =
                        RegionalTaxonStatus::update_by_taxon_id_and_region_id(taxon_id, region_id);
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
                        query = query.harvest_window(toasty::stmt::patch(
                            RegionalHarvestWindow::fields().start(),
                            harvest_start.with().year(2000).build().unwrap(),
                        ));
                    }
                    if let Some(harvest_end) = props.harvest_end {
                        query = query.harvest_window(toasty::stmt::patch(
                            RegionalHarvestWindow::fields().end(),
                            harvest_end.with().year(2000).build().unwrap(),
                        ));
                    }
                    query.exec(db).await?;
                    println!("Modified taxon {} in region {}", taxon_id, region_id);
                }
                RegionTaxaCommands::List => list_regional_taxa(db, *region_id).await?,
                RegionTaxaCommands::Remove {
                    taxon_id,
                    assumeyes,
                } => {
                    if *assumeyes
                        || inquire::Confirm::new(
                            "Are you sure you wish to remove this regional taxon?",
                        )
                        .with_default(false)
                        .prompt()?
                    {
                        RegionalTaxonStatus::delete_by_taxon_id_and_region_id(
                            db, taxon_id, region_id,
                        )
                        .await?;
                        println!("Removed taxon {} from region {}", taxon_id, region_id);
                    }
                }
            },
        }
        Ok(())
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum RegionTaxaCommands {
    #[command(about = "Print a list of taxa for a region")]
    List,
    #[command(about = "Show regional information about a taxon")]
    Show {
        #[arg(short, long, help = "A taxon ID")]
        taxon_id: u64,
    },
    #[command(about = "Add a taxon to a region")]
    Add {
        #[arg(short, long, help = "A taxon ID")]
        taxon_id: u64,
        #[command(flatten)]
        props: RegionalTaxonProperties,
    },
    #[command(about = "Modify information about a taxon within a region", group(clap::ArgGroup::new("modify_taxon_fields").args(["origin", "c_value", "conservation_status", "wetland_indicator", "harvest_start", "harvest_end"]).required(true).multiple(false)))]
    Modify {
        #[arg(short, long, help = "A taxon ID")]
        taxon_id: u64,
        #[command(flatten)]
        props: RegionalTaxonProperties,
    },
    #[command(about = "Remove a taxon from a region")]
    Remove {
        #[arg(short, long, help = "A taxon ID")]
        taxon_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}
