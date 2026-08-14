use std::{collections::HashMap, fmt::Display};

use geo::BoundingRect;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use toasty::{Db, Deferred};

use crate::{
    ImportProgressReporter, dto::ObjectReference, error::ImportExportError,
    region::file::RegionInfo, taxonomy::Taxon,
};

pub mod dto;

#[derive(
    Debug, Clone, Copy, toasty::Embed, strum::Display, clap::ValueEnum, Serialize, Deserialize,
)]
#[clap(rename_all = "kebab-case")]
pub enum WetlandIndicator {
    #[column(variant = 1)]
    #[serde(rename = "OBL")]
    ObligateWetland,
    #[column(variant = 2)]
    #[serde(rename = "FACW")]
    FacultativeWetland,
    #[column(variant = 3)]
    #[serde(rename = "FAC")]
    Facultative,
    #[column(variant = 4)]
    #[serde(rename = "FACU")]
    FacultativeUpland,
    #[column(variant = 5)]
    #[serde(rename = "UPL")]
    Upland,
    #[column(variant = 99)]
    Other,
}

#[derive(
    Debug, Clone, Copy, toasty::Embed, strum::Display, clap::ValueEnum, Serialize, Deserialize,
)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ConservationStatus {
    #[column(variant = 1)]
    Endangered,
    #[column(variant = 2)]
    Threatened,
    #[column(variant = 3)]
    SpecialConcern,
}

impl From<&Region> for crate::dto::ObjectReference {
    fn from(region: &Region) -> Self {
        Self {
            id: region.id,
            name: Some(region.name.clone()),
        }
    }
}

#[derive(
    Debug, Clone, Copy, toasty::Embed, Serialize, Deserialize, strum::Display, clap::ValueEnum,
)]
pub enum RegionCategory {
    Nation,
    Province,
    County,
    Municipality,
    Other,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Region {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub name: String,
    #[column(type = "TEXT")]
    pub geometry: Option<toasty::Json<geojson::Geometry>>,
    pub notes: Option<String>,
    #[index]
    pub category: RegionCategory,

    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,

    #[has_many]
    pub taxon_statuses: Deferred<Vec<RegionalTaxonStatus>>,
    #[has_many]
    pub npcs: Deferred<Vec<NativePlantCommunity>>,
}

impl Region {
    pub fn reference(&self) -> ObjectReference {
        self.into()
    }

    /// NOTE: assumes that `self` has all taxa loaded from the database
    pub async fn export<W>(&self, writer: W) -> Result<(), ImportExportError>
    where
        W: std::io::Write,
    {
        let regioninfo: RegionInfo = self.into();
        serde_yaml::to_writer(writer, &regioninfo)?;
        Ok(())
    }

    pub async fn import<R>(
        db: &mut toasty::Db,
        reader: R,
        reporter: &mut dyn ImportProgressReporter,
    ) -> Result<Self, ImportExportError>
    where
        R: std::io::Read,
    {
        let info: file::RegionInfo = serde_yaml::from_reader(reader)?;

        let existing = Region::filter_by_name(&info.name).exec(db).await?;
        if !existing.is_empty() {
            return Err(ImportExportError::RegionExists(info.name));
        }

        // loop through the input list and search for names from our taxonomy that
        // match the given name. Some of these input names may map to the same name in our
        // taxonomy, so we need to eliminate duplicates at the end. We do this by storing
        // the result in a hashmap by result taxon id
        let mut lookups: HashMap<u64, file::TaxonInfo> = HashMap::default();
        reporter.begin_step("Validating taxa...", info.taxa.len());
        for taxon_info in info.taxa.into_iter() {
            reporter.increment();
            let t = match taxon_info.name.parse::<u64>() {
                Ok(val) => Taxon::get_by_id(db, val).await?,
                Err(_) => Taxon::get_by_name_or_synonym(db, &taxon_info.name)
                    .await
                    .map_err(|_e| ImportExportError::NoMatchingTaxon(taxon_info.name.clone()))?,
            };
            lookups
                .entry(t.id)
                .and_modify(|existing| {
                    // if any of the lumped taxa is native, consider the whole thing native
                    if taxon_info.origin == Some(Origin::Native) {
                        existing.origin = taxon_info.origin;
                    } else if existing.origin.is_none_or(|x| x == Origin::Unknown) {
                        // any new status overrides unknown
                        existing.origin = taxon_info.origin;
                    }
                })
                .or_insert(taxon_info);
        }
        reporter.finish_step();

        // now insert all unique taxa into the region table
        let mut taxa_create = Vec::new();
        reporter.begin_step("Importing taxa...", lookups.len());
        for (id, taxon_info) in lookups.into_iter() {
            reporter.increment();
            taxa_create.push(
                RegionalTaxonStatus::create()
                    .taxon_id(id)
                    .origin(taxon_info.origin)
                    .conservation_status(taxon_info.status)
                    .wetland_indicator(taxon_info.wetland_indicator)
                    .harvest_window(RegionalHarvestWindow::default())
                    .c_value(taxon_info.c_value),
            );
        }

        let mut txn = db.transaction().await?;
        let region = Self::create()
            .name(info.name)
            .geometry(info.geometry.map(|v| v.into()))
            .category(info.category)
            .notes(info.notes)
            .taxon_statuses(taxa_create)
            .exec(&mut txn)
            .await?;

        txn.commit().await?;
        reporter.finish_step();

        Ok(region)
    }

    pub async fn get_taxa(
        &self,
        db: &mut Db,
        query: Option<&String>,
        origin: Option<OriginFilter>,
        harvest: Option<HarvestFilter>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<(u64, Vec<Taxon>), toasty::Error> {
        let mut rts_filter = RegionalTaxonStatus::fields().region_id().eq(self.id);
        let today = jiff::Zoned::now().date().day_of_year();
        if let Some(f) = harvest {
            rts_filter = rts_filter.and(match f {
                HarvestFilter::ReadyNow => window_includes(today),
                HarvestFilter::ReadyOnDoy(doy) => window_includes(doy),
                HarvestFilter::EndingSoon(days) => RegionalTaxonStatus::fields()
                    .harvest_window()
                    .end_doy()
                    .ge(today)
                    .and(
                        RegionalTaxonStatus::fields()
                            .harvest_window()
                            .end_doy()
                            .le(today + days),
                    ),
                HarvestFilter::StartingSoon(days) => RegionalTaxonStatus::fields()
                    .harvest_window()
                    .start_doy()
                    .ge(today)
                    .and(
                        RegionalTaxonStatus::fields()
                            .harvest_window()
                            .start_doy()
                            .le(today + days),
                    ),
            });
        }
        if let Some(f) = origin {
            rts_filter = rts_filter.and(match f {
                OriginFilter::NativeOnly => {
                    RegionalTaxonStatus::fields().origin().eq(Origin::Native)
                }
                OriginFilter::NonNativeOnly => {
                    RegionalTaxonStatus::fields().origin().ne(Origin::Native)
                }
            })
        }

        let mut filter = Taxon::fields().regional_statuses().any(rts_filter);
        if let Some(q) = query {
            filter = filter.and(Taxon::search_filter(q));
        }
        let mut query = Taxon::filter(filter);
        let total = query.clone().count().exec(db).await?;
        query = query
            .include(
                Taxon::fields()
                    .regional_statuses()
                    .filter(RegionalTaxonStatus::fields().region_id().eq(self.id)),
            )
            .order_by(Taxon::fields().sequence().asc());
        if let Some(limit) = limit {
            query = query.limit(limit)
        }
        if let Some(offset) = offset {
            query = query.offset(offset)
        }
        Ok((total, query.exec(db).await?))
    }

    fn geometry(&self) -> Option<geojson::Geometry> {
        self.geometry.as_ref().map(|jsonvalue| jsonvalue.0.clone())
    }

    fn bounding_box(&self) -> Result<geo::Rect, RegionGeometryError> {
        self.geometry()
            .ok_or(RegionGeometryError::NoRegionGeometry)
            .and_then(|jsongeo| {
                jsongeo
                    .try_into()
                    .map_err(|_| RegionGeometryError::InvalidRegionGeometry)
                    .and_then(|g: geo::Geometry| {
                        let bounding_rect = g.bounding_rect();
                        bounding_rect.ok_or(RegionGeometryError::InvalidRegionGeometry)
                    })
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegionGeometryError {
    #[error("Region has no defined geometry")]
    NoRegionGeometry,
    #[error("Region has invalid geometry")]
    InvalidRegionGeometry,
}

fn window_includes(doy: i16) -> toasty::stmt::Expr<bool> {
    RegionalTaxonStatus::fields()
        .harvest_window()
        .start_doy()
        .le(doy)
        .and(
            RegionalTaxonStatus::fields()
                .harvest_window()
                .end_doy()
                .ge(doy),
        )
        .or(RegionalTaxonStatus::fields()
            .harvest_window()
            .start_doy()
            .gt(RegionalTaxonStatus::fields().harvest_window().end_doy())
            .and(
                RegionalTaxonStatus::fields()
                    .harvest_window()
                    .start_doy()
                    .le(doy)
                    .or(RegionalTaxonStatus::fields()
                        .harvest_window()
                        .end_doy()
                        .ge(doy)),
            ))
}

#[derive(
    Debug,
    Clone,
    Copy,
    toasty::Embed,
    strum::Display,
    clap::ValueEnum,
    Serialize,
    Deserialize,
    PartialEq,
)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    #[column(variant = 1)]
    Native,
    #[column(variant = 2)]
    Introduced,
    #[column(variant = 3)]
    Unknown,
}

pub enum OriginFilter {
    NativeOnly,
    NonNativeOnly,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, toasty::Embed, Serialize, PartialEq)]
pub struct RegionalHarvestWindow {
    pub start_doy: Option<i16>,
    pub end_doy: Option<i16>,
}

pub enum HarvestFilter {
    ReadyNow,
    ReadyOnDoy(i16),
    EndingSoon(i16),
    StartingSoon(i16),
}

impl RegionalHarvestWindow {
    pub fn is_empty(&self) -> bool {
        self.start_doy.is_none() && self.end_doy.is_none()
    }

    pub fn start_week(&self) -> Option<i16> {
        self.start_doy.map(|doy| doy.div_euclid(7))
    }

    pub fn end_week(&self) -> Option<i16> {
        self.end_doy.map(|doy| (doy + 6).div_euclid(7))
    }

    fn start_date(&self) -> Option<String> {
        self.start_doy.and_then(|d| {
            jiff::Zoned::now()
                .date()
                .with()
                .day_of_year(d)
                .build()
                .map(|d| d.strftime("%b %d").to_string())
                .ok()
        })
    }

    fn end_date(&self) -> Option<String> {
        self.end_doy.and_then(|d| {
            jiff::Zoned::now()
                .date()
                .with()
                .day_of_year(d)
                .build()
                .map(|d| d.strftime("%b %d").to_string())
                .ok()
        })
    }

    // FIXME: error handling
    pub fn from_histogram(weeks: &[u64]) -> Option<Self> {
        if weeks.len() < 52 || weeks.len() > 53 {
            return None;
        }

        /// Circular Gaussian smoothing.
        ///
        /// sigma is measured in weeks. A sigma around 1.0–1.5 works well.
        fn gaussian_smooth(hist: &[u64], sigma: f64) -> Vec<f64> {
            let n = hist.len();
            let radius = (sigma * 3.0).ceil() as isize;

            let mut kernel = Vec::new();
            let mut norm = 0.0;

            for x in -radius..=radius {
                let w = (-0.5 * (x as f64 / sigma).powi(2)).exp();
                kernel.push(w);
                norm += w;
            }

            for w in &mut kernel {
                *w /= norm;
            }

            (0..n)
                .map(|i| {
                    let mut value = 0.0;

                    for (k, &weight) in kernel.iter().enumerate() {
                        let offset = k as isize - radius;

                        let j = ((i as isize + offset).rem_euclid(n as isize)) as usize;

                        value += hist[j] as f64 * weight;
                    }

                    value
                })
                .collect()
        }

        /// Finds the shortest contiguous interval containing at least `fraction`
        /// of the total histogram weight.
        ///
        /// The histogram is treated as circular (weeks of the year), so harvest
        /// seasons that span New Year's are handled correctly.
        fn harvest_window(weights: &[f64], fraction: f64) -> Option<RegionalHarvestWindow> {
            assert!(!weights.is_empty());
            assert!((0.0..=1.0).contains(&fraction));

            let n = weights.len();

            let total: f64 = weights.iter().sum();
            if total <= 0.0 {
                return None;
            }

            let target = total * fraction;

            // Duplicate the histogram so circular windows become linear.
            let doubled: Vec<f64> = weights.iter().chain(weights.iter()).copied().collect();

            let mut best_start = 0;
            let mut best_end = 0;
            let mut best_len = usize::MAX;

            let mut start = 0;
            let mut sum = 0.0;

            for end in 0..doubled.len() {
                sum += doubled[end];

                // Never consider windows longer than one full year.
                while end - start + 1 > n {
                    sum -= doubled[start];
                    start += 1;
                }

                while sum >= target {
                    let len = end - start;

                    if len < best_len {
                        best_len = len;
                        best_start = start;
                        best_end = end;
                    }

                    sum -= doubled[start];
                    start += 1;
                }
            }

            Some(RegionalHarvestWindow {
                // after finding the best start week and best end week, use the
                // beginning of the week for the start, and the end of the week
                // for the end
                start_doy: Some((((best_start % n) as i16) * 7 + 1).clamp(1, 365)),
                end_doy: Some((((best_end % n + 1) as i16) * 7).clamp(1, 365)),
            })
        }

        let smoothed = gaussian_smooth(weeks, 1.2);
        harvest_window(&smoothed, 0.9)
    }
}

impl Display for RegionalHarvestWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            self.start_date().as_deref().unwrap_or("?"),
            self.end_date().as_deref().unwrap_or("?"),
        )
    }
}

#[derive(Debug, Clone, toasty::Model)]
#[index(taxon_id, region_id)]
pub struct RegionalTaxonStatus {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: Deferred<Taxon>,

    #[index]
    pub region_id: u64,
    #[belongs_to(key=region_id, references=id)]
    pub region: Deferred<Region>,

    pub origin: Option<Origin>,
    // generally 0-10?
    pub c_value: Option<u64>,
    pub conservation_status: Option<ConservationStatus>,
    pub wetland_indicator: Option<WetlandIndicator>,
    pub harvest_window: RegionalHarvestWindow,
    #[index]
    pub native_plant_community_id: Option<u64>,
    #[belongs_to(key=native_plant_community_id, references=id)]
    pub native_plant_community: Deferred<NativePlantCommunity>,

    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

#[derive(thiserror::Error, Debug)]
pub enum QueryHarvestError {
    #[error(transparent)]
    Inaturalist(#[from] inaturalist::Error),
    #[error(transparent)]
    Db(#[from] toasty::Error),
    #[error("Unable to find an iNaturalist Taxon")]
    NoInaturalistTaxon,
    #[error("Insufficient observations to calculate a harvest window")]
    InsufficientSamples,
    #[error(transparent)]
    Geometry(#[from] RegionGeometryError),
}

impl RegionalTaxonStatus {
    pub async fn query_harvest_info(
        &self,
        db: &mut toasty::Db,
    ) -> Result<(usize, RegionalHarvestWindow), QueryHarvestError> {
        let taxon = self.taxon.get();
        let inat = inaturalist::Client::new()?;
        let inat_taxon = if let Some(id) = taxon.inaturalist_id {
            let taxon = inat.taxon_info(id).await?;
            Some(taxon)
        } else {
            let found = taxon.find_exact_inat_taxon(&inat).await?;
            if let Some(t) = found.as_ref() {
                Taxon::update_by_id(taxon.id)
                    .inaturalist_id(t.id)
                    .exec(db)
                    .await?;
            }
            found
        };
        let Some(inat_taxon) = inat_taxon else {
            return Err(QueryHarvestError::NoInaturalistTaxon);
        };

        self.query_harvest_window_internal(inat_taxon).await
    }

    async fn query_harvest_window_internal(
        &self,
        inat_taxon: inaturalist::Taxon,
    ) -> Result<(usize, RegionalHarvestWindow), QueryHarvestError> {
        let inat = inaturalist::Client::new()?;
        let rect = self.region.get().bounding_box()?;
        let loc = inaturalist::SearchArea::BoundingBox(rect);
        let observation_window = {
            let (total, histogram) = inat.seed_histogram(inat_taxon.id, &loc).await?;
            if total < 2 {
                return Err(QueryHarvestError::InsufficientSamples);
            }
            let harvest_window =
                RegionalHarvestWindow::from_histogram(&histogram).unwrap_or_default();
            (total, harvest_window)
        };
        Ok(observation_window)
    }
}

#[derive(Debug, Clone, toasty::Model)]
pub struct NativePlantCommunity {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub region_id: u64,
    #[belongs_to(key=region_id, references=id)]
    pub region: Deferred<Region>,

    #[index]
    pub name: String,

    #[has_many]
    regional_taxon_statuses: Deferred<Vec<RegionalTaxonStatus>>,
}

mod file {
    use serde::{Deserialize, Serialize};
    use serde_with::skip_serializing_none;

    use crate::region::{
        ConservationStatus, Origin, Region, RegionCategory, RegionalTaxonStatus, WetlandIndicator,
    };

    #[skip_serializing_none]
    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct TaxonInfo {
        pub(crate) name: String,
        pub(crate) c_value: Option<u64>,
        pub(crate) origin: Option<Origin>,
        pub(crate) status: Option<ConservationStatus>,
        pub(crate) wetland_indicator: Option<WetlandIndicator>,
    }

    impl From<&RegionalTaxonStatus> for TaxonInfo {
        fn from(value: &RegionalTaxonStatus) -> Self {
            tracing::debug!("Converting {}", value.taxon.get().complete_name);
            Self {
                name: value.taxon.get().complete_name.clone(),
                c_value: value.c_value,
                origin: value.origin,
                status: value.conservation_status,
                wetland_indicator: value.wetland_indicator,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct RegionInfo {
        pub(crate) name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) notes: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) geometry: Option<geojson::Geometry>,
        pub(crate) category: RegionCategory,
        pub(crate) taxa: Vec<TaxonInfo>,
        // npcs: Vec<NativePlantCommunityInfo>,
    }

    impl From<&Region> for RegionInfo {
        fn from(value: &Region) -> Self {
            let mut taxa = value.taxon_statuses.get().clone();
            taxa.sort_by_key(|t| t.taxon.get().sequence);
            let taxa: Vec<TaxonInfo> = taxa.into_iter().map(|ts| (&ts).into()).collect();
            Self {
                name: value.name.clone(),
                geometry: value.geometry(),
                category: value.category,
                taxa,
                notes: value.notes.clone(),
            }
        }
    }
}
