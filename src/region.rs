use std::{collections::HashMap, fmt::Display, path::Path};

use serde::{Deserialize, Serialize};
use toasty::Deferred;
use tokio::io::AsyncReadExt;

use crate::{
    ImportProgressReporter,
    taxonomy::{Synonym, Taxon},
};

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

#[derive(Debug, Clone, toasty::Model)]
pub struct Region {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub name: String,
    // FIXME: geojson??
    pub geometry: Option<toasty::Json<geojson::Geometry>>,
    pub notes: Option<String>,

    #[has_many]
    pub taxon_statuses: Deferred<Vec<RegionalTaxonStatus>>,
    #[has_many]
    pub npcs: Deferred<Vec<NativePlantCommunity>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("A region with the name '{0}' already exists")]
    RegionExists(String),
    #[error("Unable to find a taxon equivalent to '{0}' in the database")]
    NoMatchingTaxon(String),
    #[error(transparent)]
    Database(#[from] toasty::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    FileFormat(#[from] serde_yaml::Error),
}

impl Region {
    pub fn reference(&self) -> String {
        format!("{}: {}", self.id, self.name)
    }

    pub async fn import<P>(
        db: &mut toasty::Db,
        path: P,
        reporter: &mut dyn ImportProgressReporter,
    ) -> Result<Self, ImportError>
    where
        P: AsRef<Path>,
    {
        let mut f = tokio::fs::OpenOptions::new().read(true).open(path).await?;
        let mut info_string = String::new();
        f.read_to_string(&mut info_string).await?;

        let info: file::RegionInfo = serde_yaml::from_str(&info_string)?;

        let existing = Region::filter_by_name(&info.name).exec(db).await?;
        if !existing.is_empty() {
            return Err(ImportError::RegionExists(info.name));
        }

        // loop through the input list and search for names from our taxonomy that
        // match the given name. Some of these input names may map to the same name in our
        // taxonomy, so we need to eliminate duplicates at the end. We do this by storing
        // the result in a hashmap by result taxon id
        let mut lookups: HashMap<u64, file::TaxonInfo> = HashMap::default();
        reporter.begin_step("Validating taxa...", info.taxa.len());
        for taxon_info in info.taxa.into_iter() {
            reporter.increment();
            let t = find_taxon_for_name(db, &taxon_info.name).await?;
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
            .notes(info.notes)
            .taxon_statuses(taxa_create)
            .exec(&mut txn)
            .await?;

        txn.commit().await?;
        reporter.finish_step();

        Ok(region)
    }
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

#[derive(Debug, Clone, Default, toasty::Embed)]
pub struct RegionalHarvestWindow {
    pub start_doy: Option<i16>,
    pub end_doy: Option<i16>,
}

impl Display for RegionalHarvestWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            self.start_doy
                .and_then(|d| {
                    jiff::civil::Date::default()
                        .with()
                        .year(2000)
                        .day_of_year(d)
                        .build()
                        .map(|d| d.strftime("%b %d").to_string())
                        .ok()
                })
                .as_deref()
                .unwrap_or("?"),
            self.end_doy
                .and_then(|d| {
                    jiff::civil::Date::default()
                        .with()
                        .year(2000)
                        .day_of_year(d)
                        .build()
                        .map(|d| d.strftime("%b %d").to_string())
                        .ok()
                })
                .as_deref()
                .unwrap_or("?"),
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
    use crate::region::{ConservationStatus, Origin, WetlandIndicator};

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub(crate) struct TaxonInfo {
        pub(crate) name: String,
        pub(crate) c_value: Option<u64>,
        pub(crate) origin: Option<Origin>,
        pub(crate) status: Option<ConservationStatus>,
        pub(crate) wetland_indicator: Option<WetlandIndicator>,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub(crate) struct RegionInfo {
        pub(crate) name: String,
        pub(crate) geometry: Option<geojson::Geometry>,
        pub(crate) taxa: Vec<TaxonInfo>,
        pub(crate) notes: Option<String>,
        // npcs: Vec<NativePlantCommunityInfo>,
    }
}

async fn find_taxon_for_name(
    db: &mut dyn toasty::Executor,
    name: &str,
) -> Result<Taxon, ImportError> {
    Ok(match name.parse::<u64>() {
        Ok(val) => Taxon::get_by_id(db, val).await?,
        Err(_) => match Taxon::get_by_complete_name(db, name).await {
            Ok(taxon) => taxon,
            Err(_e) => {
                // tracing::warn!(?e);
                Synonym::filter_by_complete_name(name)
                    .include(Synonym::fields().taxon())
                    .one()
                    .exec(db)
                    .await
                    .map_err(|_| ImportError::NoMatchingTaxon(name.to_string()))
                    .map(|synonym| synonym.taxon.get().clone())?
            }
        },
    })
}
