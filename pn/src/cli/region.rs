use std::f64::consts::PI;
use std::fmt::Display;
use std::path::PathBuf;

use crate::cli::OutputFormat;
use crate::util::dialog::{confirm, input, select};
use crate::util::{IndicatifImportProgress, inat_taxon_for_taxon};
use crate::views::regions::{
    RegionDetailsView, RegionalHarvestDateListView, RegionalTaxonStatusDetailsView,
    RegionalTaxonStatusHarvestView, RegionsListView,
};
use crate::views::taxa::RegionalTaxaListView;
use crate::views::{JsonView, YamlView};
use anyhow::anyhow;
use demand::DemandOption;
use geo::BoundingRect;
use geo::ChamberlainDuquetteArea;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use jiff::civil::Date;
use libpropagation::region::dto::{
    CompactRegion, FullRegion, RegionalTaxonHarvestInfo, RegionalTaxonStatusDetails,
    RegionalTaxonStatusDetailsNoRegion, RegionalTaxonStatusHarvest,
};
use libpropagation::taxonomy::TaxonIdentifier;
use libpropagation::{
    region::{
        ConservationStatus, Origin, Region, RegionalHarvestWindow, RegionalTaxonStatus,
        WetlandIndicator,
    },
    taxonomy::Taxon,
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
    #[command(about = "Print a list of regions", alias = "ls")]
    List,
    #[command(about = "Show detailed information about a region")]
    Show { id: u64 },
    #[command(about = "Add a new region to the database", alias = "new")]
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
    #[command(about = "Export a region from the database")]
    Export {
        #[arg(help = "A region ID")]
        id: u64,
        #[arg(short, long, help = "A path to save the file describing the region")]
        output_file: Option<PathBuf>,
    },
    #[command(about = "Modify information about a region", group(clap::ArgGroup::new("modify_fields").args(["name", "geometry_string", "geometry_file", "notes"]).required(true).multiple(true)), alias="edit")]
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
    #[command(about = "Look up harvest dates from iNaturalist")]
    LookupHarvestDates {
        id: u64,
        #[arg(
            short,
            long,
            help = "Minimum number of observations to use for calculating the harvest window",
            default_value_t = 10
        )]
        min_samples: usize,
        #[arg(
            long,
            help = "use interactive mode",
            long_help = "In interactive mode, you may be prompted to update values. In non-interactive mode it only updates taxa that don't yet have a value set."
        )]
        interactive: bool,
        #[arg(long, help = "skip the first N taxa")]
        skip: Option<usize>,
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
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            RegionCommands::List => {
                let regions: Vec<CompactRegion> = Region::all()
                    .include(Region::fields().taxon_statuses())
                    .exec(db)
                    .await?
                    .into_iter()
                    .map(|region| region.into())
                    .collect();
                let output = match format {
                    OutputFormat::Text => RegionsListView::new(&regions).render()?,
                    OutputFormat::Json => JsonView::new(&regions).render()?,
                    OutputFormat::Yaml => YamlView::new(&regions).render()?,
                };
                println!("{output}");
            }
            RegionCommands::Show { id } => {
                let region: FullRegion = Region::filter_by_id(id)
                    .include(Region::fields().taxon_statuses())
                    .one()
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => RegionDetailsView::new(&region).render()?,
                    OutputFormat::Json => JsonView::new(&region).render()?,
                    OutputFormat::Yaml => YamlView::new(&region).render()?,
                };
                println!("{output}")
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
                let region: FullRegion = Region::filter_by_id(id)
                    .include(Region::fields().taxon_statuses())
                    .one()
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => RegionDetailsView::new(&region).render()?,
                    OutputFormat::Json => JsonView::new(&region).render()?,
                    OutputFormat::Yaml => YamlView::new(&region).render()?,
                };
                println!("{output}")
            }
            RegionCommands::Add {
                region_name,
                geometry,
                notes,
            } => {
                let new_region: FullRegion = Region::create()
                    .name(region_name)
                    .geometry(geometry.resolve().await?.map(|v| v.into()))
                    .notes(notes)
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => RegionDetailsView::new(&new_region).render()?,
                    OutputFormat::Json => JsonView::new(&new_region).render()?,
                    OutputFormat::Yaml => YamlView::new(&new_region).render()?,
                };
                println!("{output}")
            }
            RegionCommands::Import { path } => {
                let file_reader = std::fs::OpenOptions::new().read(true).open(path)?;
                let region: FullRegion =
                    Region::import(db, file_reader, &mut IndicatifImportProgress::default())
                        .await?
                        .into();
                let output = match format {
                    OutputFormat::Text => RegionDetailsView::new(&region).render()?,
                    OutputFormat::Json => JsonView::new(&region).render()?,
                    OutputFormat::Yaml => YamlView::new(&region).render()?,
                };
                println!("{output}")
            }
            RegionCommands::Export { id, output_file } => {
                let region = Region::filter_by_id(id)
                    .include(Region::fields().taxon_statuses().taxon())
                    .one()
                    .exec(db)
                    .await?;
                debug!("Got region");

                match output_file {
                    Some(path) => {
                        region
                            .export(
                                std::fs::OpenOptions::new()
                                    .write(true)
                                    .create_new(true)
                                    .open(path)?,
                            )
                            .await?;
                        println!(
                            "Exported region '{}' to '{}'",
                            region.reference(),
                            path.display()
                        );
                    }
                    None => region.export(std::io::stdout()).await?,
                };
            }
            RegionCommands::Remove { id, assumeyes } => {
                if *assumeyes
                    || {
                        let region: FullRegion = Region::filter_by_id(id)
                            .include(Region::fields().taxon_statuses())
                            .one()
                            .exec(db)
                            .await?
                            .into();
                        println!("{}", RegionDetailsView::new(&region).render()?);
                        confirm("Are you sure you wish to delete this region? All associated data will be deleted.")
                        .selected(false)
                        .run()?
                    }
                {
                    Region::delete_by_id(db, id).await?;
                    println!("Deleted region {id} from the database");
                }
            }
            RegionCommands::LookupHarvestDates {
                id: region_id,
                min_samples,
                interactive,
                skip,
            } => {
                let region = Region::filter_by_id(region_id).one().exec(db).await?;
                // the db querying is such a small part of this overall
                // algorithm, so get a full list quickly by not including any
                // child fields and then fetch the full object within the loop
                let taxa = Taxon::filter(
                    Taxon::fields()
                        .regional_statuses()
                        .any(RegionalTaxonStatus::fields().region_id().eq(region_id)),
                )
                .order_by(Taxon::fields().sequence().asc())
                .exec(db)
                .await?;
                let mut n_updates = 0;
                let total = taxa.len();
                let taxon_iter = taxa.iter().skip(skip.unwrap_or_default());
                if *interactive {
                    for (i, taxon) in taxon_iter.enumerate() {
                        let mut fullrts = RegionalTaxonStatus::filter_by_taxon_id_and_region_id(
                            taxon.id, region_id,
                        )
                        .one()
                        .include(RegionalTaxonStatus::fields().taxon().vernaculars())
                        .include(RegionalTaxonStatus::fields().region())
                        .exec(db)
                        .await?;
                        println!("{}/{total}", i + 1 + skip.unwrap_or_default());
                        if lookup_harvest_dates_interactive(db, &mut fullrts, min_samples)
                            .await
                            .is_ok()
                        {
                            n_updates += 1
                        };
                        println!();
                    }
                } else {
                    let pb = ProgressBar::new(taxa.len() as u64);
                    pb.set_style(ProgressStyle::with_template(
                        "{wide_bar} {percent}%\nQuerying '{msg}'",
                    )?);
                    pb.set_message("Preparing...");
                    for taxon in taxon_iter.progress_with(pb.clone()) {
                        let fullrts = RegionalTaxonStatus::filter_by_taxon_id_and_region_id(
                            taxon.id, region_id,
                        )
                        .one()
                        .include(RegionalTaxonStatus::fields().taxon().vernaculars())
                        .include(RegionalTaxonStatus::fields().region())
                        .exec(db)
                        .await?;
                        let taxon = fullrts.taxon.get();
                        if fullrts.harvest_window.start_doy.is_none()
                            && fullrts.harvest_window.end_doy.is_none()
                        {
                            pb.set_message(taxon.complete_name.clone());
                            let inat = inaturalist::Client::new()?;
                            let inat_taxon = if let Some(id) = taxon.inaturalist_id {
                                let taxon = inat.taxon_info(id).await?;
                                Some(taxon)
                            } else {
                                let mut found = None;
                                let possible_taxa = inat
                                    .taxon_search(&taxon.names())
                                    .await?
                                    .into_iter()
                                    .filter(|t| t.is_active)
                                    .collect::<Vec<_>>();
                                if !possible_taxa.is_empty() {
                                    for possibility in possible_taxa {
                                        if taxon.matches(&possibility) {
                                            debug!(
                                                "Using {} for {}",
                                                possibility.name,
                                                taxon.reference()
                                            );
                                            found = Some(possibility);
                                            break;
                                        }
                                    }
                                }
                                found
                            };
                            let Some(inat_taxon) = inat_taxon else {
                                continue;
                            };

                            match query_harvest_window_for_taxon(
                                min_samples,
                                inat_taxon,
                                &region,
                                false,
                            )
                            .await
                            {
                                Ok(obs_window) => {
                                    let harvest_window = RegionalHarvestWindow {
                                        start_doy: Some(obs_window.start_doy),
                                        end_doy: Some(obs_window.end_doy),
                                    };
                                    RegionalTaxonStatus::update_by_id(fullrts.id)
                                        .harvest_window(&harvest_window)
                                        .exec(db)
                                        .await?;
                                    n_updates += 1;
                                    debug!("Updated {} to {}", taxon.reference(), harvest_window)
                                }
                                Err(e) => debug!(
                                    "Failed to calculate a harvest window for {}: {e}",
                                    fullrts.taxon_id
                                ),
                            };
                        } else {
                            debug!(
                                "Skipping taxon {} as it already has harvest data",
                                fullrts.taxon_id,
                            )
                        };
                    }
                }
                println!("Updated {n_updates} taxa");
            }
            RegionCommands::Taxa { region_id, command } => {
                command.run(db, *region_id, format).await?
            }
        }
        Ok(())
    }
}

fn parse_month_day(input: &str) -> anyhow::Result<jiff::civil::Date> {
    let mut parsed = jiff::fmt::strtime::parse("%m-%d", input)?;
    parsed.set_year(Some(2000))?;
    parsed.to_date().map_err(|e| e.into())
}

#[derive(clap::Args, Debug)]
pub struct RegionalTaxonProperties {
    #[arg(long, help = "Origin of the taxon vis-a-vis this region")]
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
    #[command(about = "Print a list of taxa for a region", group(clap::ArgGroup::new("list_taxa_fields").args(["missing_dates", "ready_to_harvest"]).required(false).multiple(false)), alias="ls")]
    List {
        #[arg(
            long,
            help = "Show only species that are missing harvest date information"
        )]
        missing_dates: bool,
        #[arg(long, help = "Show only species that are ready to harvest today")]
        ready_to_harvest: bool,
        #[arg(long, help = "Show only native species")]
        native: bool,
    },
    #[command(about = "Show regional information about a taxon")]
    Show {
        #[arg(help = "A taxon name or ID")]
        name_or_id: TaxonIdentifier,
    },
    #[command(about = "Add a taxon to a region", alias = "new")]
    Add {
        #[arg(help = "A taxon name or ID")]
        name_or_id: TaxonIdentifier,
        #[command(flatten)]
        props: RegionalTaxonProperties,
    },
    #[command(about = "Modify information about a taxon within a region", group(clap::ArgGroup::new("modify_taxon_fields").args(["origin", "c_value", "conservation_status", "wetland_indicator", "harvest_start", "harvest_end"]).required(true).multiple(false)), alias="edit")]
    Modify {
        #[arg(help = "A taxon name or ID")]
        name_or_id: TaxonIdentifier,
        #[command(flatten)]
        props: RegionalTaxonProperties,
    },
    #[command(about = "Remove a taxon from a region")]
    Remove {
        #[arg(help = "A taxon name or ID")]
        name_or_id: TaxonIdentifier,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    #[command(about = "Look up harvest dates from iNaturalist")]
    LookupHarvestDates {
        #[arg(help = "A taxon name or ID")]
        name_or_id: TaxonIdentifier,
        #[arg(
            short,
            long,
            help = "Minimum number of observations to use for calculating the harvest window",
            default_value_t = 10
        )]
        min_samples: usize,
    },
}

impl RegionTaxaCommands {
    pub async fn run(
        &self,
        db: &mut Db,
        region_id: u64,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            RegionTaxaCommands::Show { name_or_id } => {
                let taxon_id = match name_or_id {
                    TaxonIdentifier::Id(id) => *id,
                    TaxonIdentifier::Name(name) => {
                        Taxon::get_by_complete_name_ignore_case(db, name).await?.id
                    }
                };
                load_and_show_regional_taxon_details(db, region_id, taxon_id, format).await?;
            }
            RegionTaxaCommands::Add { name_or_id, props } => {
                let taxon_id = match name_or_id {
                    TaxonIdentifier::Id(id) => *id,
                    TaxonIdentifier::Name(name) => {
                        Taxon::get_by_complete_name_ignore_case(db, name).await?.id
                    }
                };
                // make sure region exists
                let _r = Region::get_by_id(db, region_id).await?;
                let status: RegionalTaxonStatusDetails = RegionalTaxonStatus::create()
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
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => RegionalTaxonStatusDetailsView::new(&status).render()?,
                    OutputFormat::Json => JsonView::new(&status).render()?,
                    OutputFormat::Yaml => YamlView::new(&status).render()?,
                };
                println!("{output}");
            }
            RegionTaxaCommands::Modify { name_or_id, props } => {
                let taxon_id = match name_or_id {
                    TaxonIdentifier::Id(id) => *id,
                    TaxonIdentifier::Name(name) => {
                        Taxon::get_by_complete_name_ignore_case(db, name).await?.id
                    }
                };
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
                load_and_show_regional_taxon_details(db, region_id, taxon_id, format).await?;
            }
            RegionTaxaCommands::List {
                missing_dates,
                ready_to_harvest,
                native,
            } => {
                let day = jiff::Zoned::now().date().day_of_year();
                // include species that start harvesting in the next week
                let start = day + 7;
                // include species that finished harvesting a week ago
                let end = day - 7;
                let mut filter = RegionalTaxonStatus::fields().region_id().eq(region_id);
                filter = if *missing_dates {
                    filter.and(
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
                    )
                } else if *ready_to_harvest {
                    tracing::debug!("Finding species ready to harvest");
                    filter.and(
                        RegionalTaxonStatus::fields()
                            .harvest_window()
                            .start_doy()
                            .le(start)
                            .and(
                                RegionalTaxonStatus::fields()
                                    .harvest_window()
                                    .end_doy()
                                    .ge(end),
                            )
                            .or(RegionalTaxonStatus::fields()
                                .harvest_window()
                                .start_doy()
                                .gt(RegionalTaxonStatus::fields().harvest_window().end_doy())
                                .and(
                                    RegionalTaxonStatus::fields()
                                        .harvest_window()
                                        .start_doy()
                                        .le(start)
                                        .or(RegionalTaxonStatus::fields()
                                            .harvest_window()
                                            .end_doy()
                                            .ge(end)),
                                )),
                    )
                } else {
                    filter
                };
                filter = if *native {
                    filter.and(RegionalTaxonStatus::fields().origin().eq(Origin::Native))
                } else {
                    filter
                };
                let taxa = Taxon::filter(Taxon::fields().regional_statuses().any(filter))
                    .include(Taxon::fields().regional_statuses())
                    .order_by(Taxon::fields().sequence().asc())
                    .exec(db)
                    .await?;
                let output = if *ready_to_harvest {
                    let mut regional_taxa = taxa
                        .into_iter()
                        .filter_map(|item| {
                            item.regional_statuses
                                .get()
                                .iter()
                                .find(|&rts| rts.region_id == region_id)
                                .map(|rts| RegionalTaxonHarvestInfo {
                                    taxon: (&item).into(),
                                    harvest_window: rts.harvest_window.clone(),
                                })
                        })
                        .collect::<Vec<_>>();
                    // toasty can't sort by an expression on a column (e.g.
                    // difference between today and end day), so sort in memory
                    // after fetching
                    regional_taxa.sort_by_key(|item| {
                        (item.harvest_window.end_doy.unwrap() - end).rem_euclid(365)
                    });
                    match format {
                        OutputFormat::Text => {
                            RegionalHarvestDateListView::new(&regional_taxa).render()?
                        }
                        OutputFormat::Json => JsonView::new(&regional_taxa).render()?,
                        OutputFormat::Yaml => YamlView::new(&regional_taxa).render()?,
                    }
                } else {
                    let statuses = RegionalTaxonStatusDetailsNoRegion::from_taxa(taxa, region_id);
                    match format {
                        OutputFormat::Text => RegionalTaxaListView::new(&statuses).render()?,
                        OutputFormat::Json => JsonView::new(&statuses).render()?,
                        OutputFormat::Yaml => YamlView::new(&statuses).render()?,
                    }
                };
                println!("{output}");
            }
            RegionTaxaCommands::Remove {
                name_or_id,
                assumeyes,
            } => {
                let taxon_id = match name_or_id {
                    TaxonIdentifier::Id(id) => *id,
                    TaxonIdentifier::Name(name) => {
                        Taxon::get_by_complete_name_ignore_case(db, name).await?.id
                    }
                };
                if *assumeyes || {
                    load_and_show_regional_taxon_details(
                        db,
                        region_id,
                        taxon_id,
                        OutputFormat::Text,
                    )
                    .await?;
                    confirm("Are you sure you wish to remove this taxon from the region?")
                        .selected(false)
                        .run()?
                } {
                    RegionalTaxonStatus::delete_by_taxon_id_and_region_id(db, taxon_id, region_id)
                        .await?;
                    println!("Removed taxon {} from region {}", taxon_id, region_id);
                }
            }
            Self::LookupHarvestDates {
                name_or_id,
                min_samples,
            } => {
                let taxon_id = match name_or_id {
                    TaxonIdentifier::Id(id) => *id,
                    TaxonIdentifier::Name(name) => {
                        Taxon::get_by_complete_name_ignore_case(db, name).await?.id
                    }
                };
                let mut rts =
                    RegionalTaxonStatus::filter_by_taxon_id_and_region_id(taxon_id, region_id)
                        .include(RegionalTaxonStatus::fields().taxon().vernaculars())
                        .include(RegionalTaxonStatus::fields().region())
                        .one()
                        .exec(db)
                        .await?;
                lookup_harvest_dates_interactive(db, &mut rts, min_samples).await?;
            }
        }
        Ok(())
    }
}

async fn lookup_harvest_dates_interactive(
    db: &mut Db,
    rts: &mut RegionalTaxonStatus,
    min_samples: &usize,
) -> Result<bool, anyhow::Error> {
    let mut updated = false;
    let taxon = rts.taxon.get();
    let region = rts.region.get();
    let dto: RegionalTaxonStatusHarvest = rts.clone().into();
    println!("{}", RegionalTaxonStatusHarvestView::new(&dto).render()?);
    let inat = inaturalist::Client::new()?;
    let inat_taxon = if let Some(id) = taxon.inaturalist_id {
        let taxon = inat.taxon_info(id).await?;
        println!("Using iNaturalist taxon '{}'", taxon);
        taxon
    } else {
        let inat_taxon = inat_taxon_for_taxon(taxon, &inat).await?;
        Taxon::update_by_id(taxon.id)
            .inaturalist_id(inat_taxon.id)
            .exec(db)
            .await?;
        inat_taxon
    };
    let observation_window =
        query_harvest_window_for_taxon(min_samples, inat_taxon, region, true).await?;
    let window = RegionalHarvestWindow {
        start_doy: Some(observation_window.start_doy),
        end_doy: Some(observation_window.end_doy),
    };
    if window != rts.harvest_window {
        println!(
            "Based on {} samples, the harvest window for '{}' in region '{}' is [{}]. ",
            observation_window.nsamples,
            taxon.reference(),
            region.reference(),
            window
        );
        if confirm(&format!(
            "Update database? Current harvest window: {}",
            rts.harvest_window,
        ))
        .selected(false)
        .run()?
        {
            rts.update().harvest_window(window).exec(db).await?;
            updated = true;
        }
    } else {
        println!("Database value already matches calculated harvest window");
    }
    println!();
    Ok(updated)
}

async fn load_and_show_regional_taxon_details(
    db: &mut Db,
    region_id: u64,
    taxon_id: u64,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let status: RegionalTaxonStatusDetails =
        RegionalTaxonStatus::filter_by_taxon_id_and_region_id(taxon_id, region_id)
            .include(RegionalTaxonStatus::fields().region())
            .include(RegionalTaxonStatus::fields().taxon())
            .one()
            .exec(db)
            .await?
            .into();
    let output = match format {
        OutputFormat::Text => RegionalTaxonStatusDetailsView::new(&status).render()?,
        OutputFormat::Json => JsonView::new(&status).render()?,
        OutputFormat::Yaml => YamlView::new(&status).render()?,
    };
    println!("{output}");
    Ok(())
}

async fn query_harvest_window_for_taxon(
    min_samples: &usize,
    inat_taxon: inaturalist::Taxon,
    region: &Region,
    allow_expansion: bool,
) -> Result<ObservationWindow, anyhow::Error> {
    let inat = inaturalist::Client::new()?;
    let bounding_box = match &region.geometry {
        Some(value) => {
            let geom: geo::Geometry = value.value.clone().try_into()?;
            geom.bounding_rect()
        }
        None => None,
    };
    let loc = match bounding_box {
        Some(rect) => inaturalist::SearchArea::BoundingBox(rect),
        None => {
            let options = inat
                .place_search(
                    &input("Search for a place on inaturalist that represents this region:")
                        .run()?,
                )
                .await?;
            let selected = select("Please select one of the following iNaturalist places:")
                .options(options.into_iter().map(DemandOption::new).collect())
                .run()?;
            inaturalist::SearchArea::Place(selected.id)
        }
    };
    let observation_window = if allow_expansion {
        seed_observation_window_with_expansion(&inat, inat_taxon, loc, *min_samples).await?
    } else {
        let observations_doy = inat
            .seed_observations(inat_taxon.id, &loc)
            .await?
            .into_iter()
            .filter_map(|ob| ob.observed_on.map(|d| d.day_of_year()))
            .collect::<Vec<_>>();
        if observations_doy.len() < *min_samples {
            return Err(anyhow!(
                "Not enough observations to calculate a harvest window"
            ));
        }
        let (start, end) = calculate_harvest_window(&observations_doy).await?;
        ObservationWindow {
            start_doy: start,
            end_doy: end,
            nsamples: observations_doy.len(),
        }
    };
    Ok(observation_window)
}

async fn calculate_harvest_window(
    observations_doy: &[i16],
) -> Result<(i16, i16), inaturalist::Error> {
    if observations_doy.is_empty() {
        return Err(inaturalist::Error::InsufficientObservations(0));
    }

    let total_count = observations_doy.len();
    // 2. CALCULATE CIRCULAR MEAN
    let mut sum_sin = 0.0;
    let mut sum_cos = 0.0;

    for &day in observations_doy {
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
    UseParentTaxon,
    Calculate,
}

impl Display for MinimumObservationsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            MinimumObservationsAction::ExpandSearch => "Expand search area",
            MinimumObservationsAction::UseParentTaxon => "Use parent taxon",
            MinimumObservationsAction::Calculate => {
                "Calculate harvest window with the existing observations"
            }
        };
        write!(f, "{msg}")
    }
}

#[tokio::test]
async fn test_seed_observation_window_with_expansion() {
    tracing_subscriber::fmt::init();
    let mut db = libpropagation::db().await.unwrap();
    let taxon = Taxon::filter_by_complete_name("Polemonium reptans var. reptans")
        .include(Taxon::fields().vernaculars())
        .one()
        .exec(&mut db)
        .await
        .unwrap();
    let client = inaturalist::Client::new().unwrap();
    let inat_taxon = inat_taxon_for_taxon(&taxon, &client).await.unwrap();
    let mn_bbox = geo::Rect::new(
        geo::coord! { x: -97.239651, y: 43.499269 }, // Southwest corner
        geo::coord! { x: -89.490365, y: 49.384687 }, // Northeast corner
    );
    let _obs = seed_observation_window_with_expansion(
        &client,
        inat_taxon,
        inaturalist::SearchArea::BoundingBox(mn_bbox),
        10,
    )
    .await
    .unwrap();
}

async fn seed_observation_window_with_expansion(
    client: &inaturalist::Client,
    taxon: inaturalist::Taxon,
    mut loc: inaturalist::SearchArea,
    min_samples: usize,
) -> anyhow::Result<ObservationWindow> {
    tracing::debug!(?taxon, "getting seed observations with expansion");
    let mut taxon = taxon;
    loop {
        let observations_doy = client
            .seed_observations(taxon.id, &loc)
            .await?
            .into_iter()
            .filter_map(|ob| ob.observed_on.map(|d| d.day_of_year()))
            .collect::<Vec<_>>();

        if observations_doy.len() < min_samples {
            let (msg, options) = match observations_doy.len() {
                0 => (
                    "No observations with seeds found in the current search area.".to_string(),
                    if taxon.parent_id.is_some() {
                        vec![
                            MinimumObservationsAction::ExpandSearch,
                            MinimumObservationsAction::UseParentTaxon,
                        ]
                    } else {
                        vec![MinimumObservationsAction::ExpandSearch]
                    },
                ),
                n => (
                    format!(
                        "Too few obervations with seeds found in the current search area ({n}/{min_samples})."
                    ),
                    if n < 2 {
                        vec![
                            MinimumObservationsAction::ExpandSearch,
                            MinimumObservationsAction::UseParentTaxon,
                        ]
                    } else {
                        if taxon.parent_id.is_some() {
                            vec![
                                MinimumObservationsAction::ExpandSearch,
                                MinimumObservationsAction::UseParentTaxon,
                                MinimumObservationsAction::Calculate,
                            ]
                        } else {
                            vec![
                                MinimumObservationsAction::ExpandSearch,
                                MinimumObservationsAction::Calculate,
                            ]
                        }
                    },
                ),
            };
            if let Ok(selected) = select(&msg)
                .options(options.into_iter().map(DemandOption::new).collect())
                .run()
            {
                match selected {
                    MinimumObservationsAction::ExpandSearch => {
                        let newloc = match &loc {
                            inaturalist::SearchArea::Place(_) => todo!(),
                            inaturalist::SearchArea::BoundingBox(rect) => {
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
                                let old_area = rect.chamberlain_duquette_unsigned_area()
                                    / (M_PER_KM * M_PER_KM);
                                let new_area = newrect.chamberlain_duquette_unsigned_area()
                                    / (M_PER_KM * M_PER_KM);
                                println!(
                                    "Expanding search area from {old_area:.1} km^2 to {new_area:.2} km^2",
                                );
                                inaturalist::SearchArea::BoundingBox(newrect)
                            }
                        };
                        loc = newloc;
                        continue;
                    }
                    MinimumObservationsAction::UseParentTaxon => {
                        if let Some(parent_id) = taxon.parent_id {
                            taxon = client.taxon_info(parent_id).await?;
                            println!("Using inaturalist taxon '{}'", taxon);
                            continue;
                        } else {
                            return Err(anyhow!("Unable to find parent taxon"));
                        }
                    }
                    MinimumObservationsAction::Calculate => (),
                }
            } else {
                return Err(anyhow!(
                    "Not enough observations to calculate a harvest window"
                ));
            }
        }
        let (start, end) = calculate_harvest_window(&observations_doy).await?;
        debug!(
            "Harvest dates for {}: {} - {}",
            taxon.id,
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
            nsamples: observations_doy.len(),
        });
    }
}
