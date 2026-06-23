use std::f64::consts::PI;
use std::fmt::Display;
use std::path::PathBuf;

use crate::util::IndicatifImportProgress;
use crate::{cli::print_regional_taxa_table, style};
use anyhow::anyhow;
use geo::BoundingRect;
use geo::ChamberlainDuquetteArea;
use jiff::civil::Date;
use libpropagation::taxonomy::Taxon;
use libpropagation::{
    inaturalist::{self, InaturalistTaxon, ObservationDate, SearchArea},
    region::{
        ConservationStatus, Origin, Region, RegionalHarvestWindow, RegionalTaxonStatus,
        WetlandIndicator,
    },
};
use toasty::Db;
use tracing::debug;
use tracing::trace;

#[derive(clap::Args, Debug)]
#[group(required = false, multiple = false)]
pub struct GeometryArg {
    #[arg(
        long,
        help = "path to a geojson file whose contents represent the geometry of the region",
        conflicts_with = "geometry_string"
    )]
    pub geometry_file: Option<PathBuf>,
    #[arg(
        short,
        long = "geometry",
        help = "geojson string representing the geometry of the region",
        conflicts_with = "geometry_file"
    )]
    pub geometry_string: Option<geojson::Geometry>,
}

impl GeometryArg {
    pub async fn resolve(&self) -> anyhow::Result<Option<geojson::Geometry>> {
        match (self.geometry_string.as_ref(), self.geometry_file.as_ref()) {
            (Some(geometry_string), None) => Ok(Some(geometry_string.clone())),
            (None, Some(geometry_file)) => {
                let s = tokio::fs::read_to_string(geometry_file).await?;
                Ok(Some(s.parse()?))
            }
            (None, None) => Ok(None),
            _ => Err(anyhow!(
                "Only one of 'geometry' or 'geometry_file' can be specified at the same time"
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
        geometry: GeometryArg,
        #[arg(long, help = "Free-form notes about the region")]
        notes: Option<String>,
    },
    #[command(about = "Import a new region to the database")]
    Import {
        #[arg(help = "A path to a yaml file describing a region")]
        path: PathBuf,
    },
    #[command(about = "Modify information about a region", group(clap::ArgGroup::new("modify_fields").args(["name", "geometry_string", "geometry_file", "notes"]).required(true).multiple(true)))]
    Modify {
        id: u64,
        #[command(flatten)]
        geometry: GeometryArg,
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
                    println!("{}", tbuilder.build().with(style::ListTable));
                }
            }
            RegionCommands::Show { id } => {
                let mut table = region_details_table(db, id).await?;
                println!("{}", table.with(style::DetailTable))
            }
            RegionCommands::Modify {
                id,
                geometry,
                name,
                notes,
            } => {
                let mut update_query = Region::update_by_id(id);
                let geometry = geometry.resolve().await?;
                if let Some(name) = name {
                    update_query = update_query.name(name);
                }
                if let Some(geometry) = geometry {
                    update_query = update_query.geometry(Some(geometry.into()));
                }
                if let Some(notes) = notes {
                    update_query = update_query.notes(notes);
                }
                update_query.exec(db).await?;
                println!("Region {id} updated");
            }
            RegionCommands::Add {
                region_name,
                geometry,
                notes,
            } => {
                let new_region = Region::create()
                    .name(region_name)
                    .geometry(geometry.resolve().await?.map(|v| v.into()))
                    .notes(notes)
                    .exec(db)
                    .await?;
                println!("Added new region {}", new_region.reference());
            }
            RegionCommands::Import { path } => {
                let region =
                    Region::import(db, path, &mut IndicatifImportProgress::default()).await?;
                println!(
                    "Created region '{}' with {} taxa",
                    region.reference(),
                    region.taxon_statuses.get().len()
                );
            }
            RegionCommands::Remove { id, assumeyes } => {
                if *assumeyes || {
                    println!(
                        "{}",
                        region_details_table(db, id).await?.with(style::DetailTable)
                    );
                    inquire::Confirm::new("Are you sure you wish to delete this region?")
                        .with_default(false)
                        .with_help_message("All associated data will be deleted")
                        .prompt()?
                } {
                    Region::delete_by_id(db, id).await?;
                    println!("Deleted region {id} from the database");
                }
            }
            RegionCommands::Taxa { region_id, command } => command.run(db, *region_id).await?,
        }
        Ok(())
    }
}

async fn region_details_table(db: &mut Db, id: &u64) -> Result<tabled::Table, anyhow::Error> {
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
        "Geometry",
        {
            region.geometry.map(|v| match &v.value {
                geojson::GeometryValue::Point { coordinates } => {
                    format!("Point: ({}, {})", coordinates[0], coordinates[1])
                }
                geojson::GeometryValue::LineString { coordinates } => {
                    format!("LineString: {} coordinates", coordinates.len())
                }
                geojson::GeometryValue::Polygon { coordinates } => {
                    format!("Polygon: {} linear rings", coordinates.len())
                }
                geojson::GeometryValue::MultiPoint { coordinates } => {
                    format!("MultiPoint: {} points", coordinates.len())
                }
                geojson::GeometryValue::MultiLineString { coordinates } => {
                    format!("MultiLineString: {} lines", coordinates.len())
                }
                geojson::GeometryValue::MultiPolygon { coordinates } => {
                    format!("MultiPolygon: {} polygons", coordinates.len())
                }
                geojson::GeometryValue::GeometryCollection { geometries } => {
                    format!("GeometryCollection: {} sub-geometries", geometries.len())
                }
            })
        }
        .as_deref()
        .unwrap_or("-"),
    ]);
    Ok(tbuilder.build())
}
fn parse_month_day(input: &str) -> anyhow::Result<jiff::civil::Date> {
    let mut parsed = jiff::fmt::strtime::parse("%m-%d", input)?;
    parsed.set_year(Some(2000))?;
    parsed.to_date().map_err(|e| e.into())
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
    #[command(about = "Look up harvest dates from iNaturalist")]
    LookupHarvestDates {
        #[arg(short, long, help = "A taxon ID")]
        taxon_id: u64,
        #[arg(
            short,
            long,
            help = "Minimum number of observations to use for calculating the harvest window",
            default_value_t = 10
        )]
        min_samples: usize,
    },
    #[command(about = "Show species ready to harvest on a certain date")]
    ReadyToHarvest {
        #[arg(
            short,
            long,
            help = "A date in YYYY-MM-DD format",
            default_value_t = jiff::Zoned::now().date()
        )]
        date: Date,
    },
    #[command(about = "Show species that do not have harvest dates for this region")]
    MissingDates,
}

async fn regional_taxa_status_details_table(
    db: &mut Db,
    region_id: u64,
    taxon_id: u64,
) -> Result<tabled::Table, anyhow::Error> {
    let status = RegionalTaxonStatus::filter_by_taxon_id_and_region_id(taxon_id, region_id)
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
            .unwrap_or(libpropagation::region::Origin::Unknown)
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
    Ok(tbuilder.build())
}

impl RegionTaxaCommands {
    pub async fn run(&self, db: &mut Db, region_id: u64) -> anyhow::Result<()> {
        match self {
            RegionTaxaCommands::Show { taxon_id } => {
                let mut table =
                    regional_taxa_status_details_table(db, region_id, *taxon_id).await?;
                println!("{}", table.with(style::DetailTable));
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
                        start_doy: props.harvest_start.map(|d| d.day_of_year()),
                        end_doy: props.harvest_end.map(|d| d.day_of_year()),
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
                        RegionalHarvestWindow::fields().start_doy(),
                        harvest_start.day_of_year(),
                    ));
                }
                if let Some(harvest_end) = props.harvest_end {
                    query = query.harvest_window(toasty::stmt::patch(
                        RegionalHarvestWindow::fields().end_doy(),
                        harvest_end.day_of_year(),
                    ));
                }
                query.exec(db).await?;
                println!("Modified taxon {} in region {}", taxon_id, region_id);
            }
            RegionTaxaCommands::List => {
                let regional_statuses = RegionalTaxonStatus::filter(
                    RegionalTaxonStatus::fields().region_id().eq(region_id),
                )
                // FIXME: We want to order by a taxon sequence, but
                // toasty doesn't yet support ordering by data in a relation
                .exec(db)
                .await?;
                print_regional_taxa_table(db, regional_statuses).await?;
            }
            RegionTaxaCommands::Remove {
                taxon_id,
                assumeyes,
            } => {
                if *assumeyes || {
                    let mut table =
                        regional_taxa_status_details_table(db, region_id, *taxon_id).await?;
                    println!("{}", table.with(style::DetailTable));
                    inquire::Confirm::new(
                        "Are you sure you wish to remove this taxon from the region? ",
                    )
                    .with_default(false)
                    .prompt()?
                } {
                    RegionalTaxonStatus::delete_by_taxon_id_and_region_id(db, taxon_id, region_id)
                        .await?;
                    println!("Removed taxon {} from region {}", taxon_id, region_id);
                }
            }
            Self::LookupHarvestDates {
                taxon_id,
                min_samples,
            } => {
                let mut rts =
                    RegionalTaxonStatus::filter_by_taxon_id_and_region_id(taxon_id, region_id)
                        .include(RegionalTaxonStatus::fields().taxon().vernaculars())
                        .include(RegionalTaxonStatus::fields().region())
                        .one()
                        .exec(db)
                        .await?;
                let taxon = rts.taxon.get();
                let region = rts.region.get();
                println!(
                    "Looking up observations of '{}' with seed annotations within region '{}' at iNaturalist.org",
                    taxon.reference(),
                    region.reference(),
                );
                let client = inaturalist::client()?;
                let inat_id = if let Some(id) = taxon.inaturalist_id {
                    id
                } else {
                    let id = inat_id_for_taxon(taxon, &client).await?;
                    Taxon::update_by_id(taxon.id)
                        .inaturalist_id(id)
                        .exec(db)
                        .await?;
                    id
                };
                let bounding_box = match &region.geometry {
                    Some(value) => {
                        let geom: geo::Geometry = value.value.clone().try_into()?;
                        geom.bounding_rect()
                    }
                    None => None,
                };
                let loc = match bounding_box {
                    Some(rect) => SearchArea::BoundingBox(rect),
                    None => {
                        let options = inaturalist::places_search(
                            &client,
                            &inquire::Text::new(
                                "Search for a place on inaturalist that represents this region:",
                            )
                            .prompt()?,
                        )
                        .await?;
                        let selected = inquire::Select::new(
                            "Please select one of the following iNaturalist places:",
                            options,
                        )
                        .prompt()?;
                        SearchArea::Place(selected.id)
                    }
                };
                let observation_window =
                    seed_observation_window_with_expansion(client, inat_id, loc, *min_samples)
                        .await?;
                let window = RegionalHarvestWindow {
                    start_doy: Some(observation_window.start_doy),
                    end_doy: Some(observation_window.end_doy),
                };
                println!(
                    "Based on {} samples, the harvest window for '{}' in region '{}' is [{}]. ",
                    observation_window.nsamples,
                    taxon.reference(),
                    region.reference(),
                    window
                );
                if inquire::Confirm::new("Update database?")
                    .with_default(false)
                    .with_help_message(&format!("Current harvest window: {}", rts.harvest_window))
                    .prompt()?
                {
                    rts.update().harvest_window(window).exec(db).await?;
                }
            }
            Self::ReadyToHarvest { date } => {
                let window = RegionalTaxonStatus::fields().harvest_window();
                let day = date.day_of_year();
                let expr = RegionalTaxonStatus::fields()
                    .region_id()
                    .eq(region_id)
                    .and(window.start_doy().le(day).and(window.end_doy().ge(day)))
                    .or(window
                        .start_doy()
                        .gt(window.end_doy())
                        .and(window.start_doy().le(day).or(window.end_doy().ge(day))));
                let regional_taxa = RegionalTaxonStatus::filter(expr)
                    .include(RegionalTaxonStatus::fields().taxon())
                    .include(RegionalTaxonStatus::fields().region())
                    .exec(db)
                    .await?;

                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["Region", "Taxon", "Harvest Dates"]);
                for regional_taxon in regional_taxa {
                    tbuilder.push_record([
                        regional_taxon.region.get().reference(),
                        regional_taxon.taxon.get().reference(),
                        regional_taxon.harvest_window.to_string(),
                    ])
                }
                println!("{}", tbuilder.build().with(style::ListTable));
            }
            Self::MissingDates => {
                // FIXME: We want to order by a taxon sequence, but
                // toasty doesn't yet support ordering by data in a relation
                let taxa = RegionalTaxonStatus::filter(
                    RegionalTaxonStatus::fields().region_id().eq(region_id).and(
                        RegionalTaxonStatus::fields()
                            .harvest_window()
                            .start_doy()
                            .is_none()
                            .and(
                                RegionalTaxonStatus::fields()
                                    .harvest_window()
                                    .end_doy()
                                    .is_none(),
                            ),
                    ),
                )
                .exec(db)
                .await?;
                print_regional_taxa_table(db, taxa).await?;
            }
        }
        Ok(())
    }
}

async fn inat_id_for_taxon(taxon: &Taxon, client: &reqwest::Client) -> anyhow::Result<u64> {
    if let Ok(inat_taxon) = inat_taxon_for_query(client, &taxon.names()).await {
        Ok(inat_taxon.id)
    } else {
        println!(
            "Couldn't find a matching taxon for the scientific name '{}'",
            taxon.complete_name
        );
        let vernaculars = taxon.vernaculars.get();
        if !vernaculars.is_empty() {
            println!("Attempting to find a match by common name...");
            for vn in taxon.vernaculars.get() {
                if let Ok(inat_taxon) = inat_taxon_for_query(client, &vn.name).await {
                    println!(
                        "Using inaturalist taxon '{} ({})'",
                        inat_taxon.name, inat_taxon.rank
                    );
                    return Ok(inat_taxon.id);
                }
                println!(
                    "Couldn't find a matching taxon for the common name '{}'",
                    vn.name
                );
            }
        }
        Err(anyhow!(
            "Unable to find a match for '{}' in iNaturalist",
            taxon.reference()
        ))
    }
}

async fn calculate_harvest_window(
    observations: &[ObservationDate],
) -> Result<(i16, i16), inaturalist::Error> {
    if observations.is_empty() {
        return Err(inaturalist::Error::InsufficientObservations(0));
    }
    let observations_doy: Vec<i16> = observations
        .iter()
        .filter_map(|ob| ob.observed_on.map(|d| d.day_of_year()))
        .collect();

    let total_count = observations_doy.len();
    // 2. CALCULATE CIRCULAR MEAN
    let mut sum_sin = 0.0;
    let mut sum_cos = 0.0;

    for &day in &observations_doy {
        let angle = (day as f64 / 365.25) * 2.0 * PI;
        sum_sin += angle.sin();
        sum_cos += angle.cos();
    }

    let avg_sin = sum_sin / total_count as f64;
    let avg_cos = sum_cos / total_count as f64;

    // REPAIRED DIRECTIONAL ANGLE CALCULATION
    let mut mean_angle = avg_sin.atan2(avg_cos);
    if mean_angle < 0.0 {
        mean_angle += 2.0 * PI; // Safely normalizes standard negative radians to 0..2*PI range
    }
    let mean_day = ((mean_angle / (2.0 * PI)) * 365.25).round() as i16 % 365;

    let r = (avg_sin.powi(2) + avg_cos.powi(2)).sqrt();
    let r_clamped = r.clamp(0.001, 1.0);
    let circ_std_dev_radians = (-2.0 * r_clamped.ln()).sqrt();
    let std_dev_days = (circ_std_dev_radians / (2.0 * PI)) * 365.25;

    // Use a conservative threshold factor (1.25 to 1.5 dev standard bounds)
    let threshold_days = (std_dev_days * 1.25).max(14.0);

    trace!("Data Center: Day {}", mean_day);
    trace!("Data Clustering Strength (R): {:.2}", r);
    trace!("Calculated Standard Deviation: {:.1} days", std_dev_days);
    trace!(
        "Filtering out entries further than {:.1} days from center...",
        threshold_days
    );

    // 4. FILTER OUTLIERS USING CIRCULAR DISTANCE
    let mut valid_days: Vec<i16> = observations_doy
        .iter()
        .copied()
        .filter(|&day| {
            let diff = (day - mean_day).abs();
            let circular_distance = diff.min(365 - diff) as f64;
            circular_distance <= threshold_days
        })
        .collect();

    if valid_days.is_empty() {
        debug!("All observations filtered out as statistical noise.");
        return Err(inaturalist::Error::InsufficientObservations(0));
    }

    // 5. CORRECT CHRONOLOGICAL SORTING (WINTER-SAFE)
    let anchor_day = (mean_day + 182) % 365;

    valid_days.sort_by_key(|&day| {
        if day > anchor_day {
            day - anchor_day
        } else {
            (day + 365) - anchor_day
        }
    });

    Ok((valid_days[0], valid_days[valid_days.len() - 1]))
}

struct ObservationWindow {
    pub start_doy: i16,
    pub end_doy: i16,
    pub nsamples: usize,
}

enum MinimumObservationsAction {
    ExpandSearch,
    Calculate,
    Abort,
}

impl Display for MinimumObservationsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            MinimumObservationsAction::ExpandSearch => "Expand search area",
            MinimumObservationsAction::Calculate => {
                "Calculate harvest window with the existing observations"
            }
            MinimumObservationsAction::Abort => "Cancel",
        };
        write!(f, "{msg}")
    }
}

async fn seed_observation_window_with_expansion(
    client: reqwest::Client,
    taxon_id: u64,
    mut loc: SearchArea,
    min_samples: usize,
) -> anyhow::Result<ObservationWindow> {
    loop {
        let observations = inaturalist::fetch_seed_observations(&client, taxon_id, &loc).await?;
        if observations.len() < min_samples {
            let (msg, options) = match observations.len() {
                0 => (
                    "No observations with seeds found in the current search area.".to_string(),
                    vec![
                        MinimumObservationsAction::ExpandSearch,
                        MinimumObservationsAction::Abort,
                    ],
                ),
                n => (
                    format!(
                        "Too few obervations with seeds found in the current search area ({n}/{min_samples})."
                    ),
                    if n < 2 {
                        vec![
                            MinimumObservationsAction::ExpandSearch,
                            MinimumObservationsAction::Abort,
                        ]
                    } else {
                        vec![
                            MinimumObservationsAction::ExpandSearch,
                            MinimumObservationsAction::Calculate,
                            MinimumObservationsAction::Abort,
                        ]
                    },
                ),
            };
            let action = inquire::Select::new(&msg, options).prompt()?;
            match action {
                MinimumObservationsAction::ExpandSearch => {
                    let newloc = match &loc {
                        SearchArea::Place(_) => todo!(),
                        SearchArea::BoundingBox(rect) => {
                            let mut newrect = *rect;
                            newrect.set_min(geo::Coord {
                                x: rect.min().x - rect.width() / 10.0,
                                y: rect.min().y - rect.height() / 10.0,
                            });
                            newrect.set_max(geo::Coord {
                                x: rect.max().x + rect.width() / 10.0,
                                y: rect.max().y + rect.height() / 10.0,
                            });
                            // Chamberlain Duquette area gives square meters
                            const M_PER_KM: f64 = 1000.0;
                            let old_area =
                                rect.chamberlain_duquette_unsigned_area() / (M_PER_KM * M_PER_KM);
                            let new_area = newrect.chamberlain_duquette_unsigned_area()
                                / (M_PER_KM * M_PER_KM);
                            println!(
                                "Expanding search area from {old_area:.1} km^2 to {new_area:.2} km^2",
                            );
                            SearchArea::BoundingBox(newrect)
                        }
                    };
                    loc = newloc;
                    continue;
                }
                MinimumObservationsAction::Calculate => (),
                MinimumObservationsAction::Abort => {
                    return Err(anyhow!(
                        "Not enough observations to calculate a harvest window"
                    ));
                }
            }
        }
        let (start, end) = calculate_harvest_window(&observations).await?;
        debug!(
            "Harvest dates for {}: {} - {}",
            taxon_id,
            Date::default()
                .with()
                .day_of_year(start)
                .build()?
                .strftime("%b-%d"),
            Date::default()
                .with()
                .day_of_year(end)
                .build()?
                .strftime("%b-%d")
        );
        break Ok(ObservationWindow {
            start_doy: start,
            end_doy: end,
            nsamples: observations.len(),
        });
    }
}

async fn inat_taxon_for_query(
    client: &reqwest::Client,
    query: &str,
) -> anyhow::Result<InaturalistTaxon> {
    let mut possible_taxa = inaturalist::find_taxon(client, query)
        .await?
        .into_iter()
        .filter(|t| t.is_active)
        .collect::<Vec<_>>();
    let taxon = if possible_taxa.len() > 1 {
        inquire::Select::new(
            &format!(
                "Please select an iNaturalist taxon that matches '{}'",
                query
            ),
            possible_taxa,
        )
        .prompt()?
    } else if possible_taxa.len() == 1 {
        possible_taxa.pop().unwrap()
    } else {
        return Err(anyhow!("Unable to find an iNaturalist taxon for '{query}'",));
    };
    Ok(taxon)
}
