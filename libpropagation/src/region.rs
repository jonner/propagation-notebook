use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use toasty::Deferred;

use crate::{
    ImportProgressReporter, error::ImportExportError, region::file::RegionInfo, taxonomy::Taxon,
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

impl From<&Region> for crate::dto::ObjectReference {
    fn from(region: &Region) -> Self {
        Self {
            id: region.id,
            name: region.name.clone(),
        }
    }
}

pub mod dto {
    use serde::Serialize;

    use crate::dto::ObjectReference;

    #[derive(Debug, Serialize)]
    pub struct CompactRegion {
        pub id: u64,
        pub name: String,
        pub n_taxa: Option<usize>,
    }

    impl From<super::Region> for CompactRegion {
        fn from(region: super::Region) -> Self {
            Self {
                id: region.id,
                name: region.name,
                n_taxa: match region.taxon_statuses.is_unloaded() {
                    true => None,
                    false => Some(region.taxon_statuses.get().len()),
                },
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct FullRegion {
        pub id: u64,
        pub name: String,
        pub notes: Option<String>,
        pub n_taxa: Option<usize>,
        pub geometry: Option<geojson::Geometry>,
    }

    impl From<super::Region> for FullRegion {
        fn from(region: super::Region) -> Self {
            Self {
                id: region.id,
                name: region.name,
                notes: region.notes,
                geometry: region.geometry.map(|inner| inner.0),
                n_taxa: match region.taxon_statuses.is_unloaded() {
                    true => None,
                    false => Some(region.taxon_statuses.get().len()),
                },
            }
        }
    }

    #[derive(Serialize)]
    pub struct RegionalTaxonHarvestInfo {
        pub taxon: ObjectReference,
        pub region: ObjectReference,
        pub harvest_window: super::RegionalHarvestWindow,
    }

    impl From<super::RegionalTaxonStatus> for RegionalTaxonHarvestInfo {
        fn from(rts: super::RegionalTaxonStatus) -> Self {
            Self {
                taxon: rts.taxon.get().into(),
                region: rts.region.get().into(),
                harvest_window: rts.harvest_window,
            }
        }
    }
}

#[skip_serializing_none]
#[serde_with::apply( Deferred => #[serde(skip_serializing_if = "Deferred::is_unloaded")])]
#[derive(Debug, Clone, toasty::Model, Serialize)]
pub struct Region {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub name: String,
    // FIXME: geojson??
    #[serde(skip)]
    pub geometry: Option<toasty::Json<geojson::Geometry>>,
    pub notes: Option<String>,

    #[has_many]
    pub taxon_statuses: Deferred<Vec<RegionalTaxonStatus>>,
    #[has_many]
    pub npcs: Deferred<Vec<NativePlantCommunity>>,
}

impl Region {
    pub fn reference(&self) -> String {
        format!("{}: {}", self.id, self.name)
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
                Err(_) => Taxon::find_by_name_or_synonym(db, &taxon_info.name)
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

#[skip_serializing_none]
#[derive(Debug, Clone, Default, toasty::Embed, Serialize)]
pub struct RegionalHarvestWindow {
    pub start_doy: Option<i16>,
    pub end_doy: Option<i16>,
}

impl RegionalHarvestWindow {
    pub fn is_empty(&self) -> bool {
        self.start_doy.is_none() && self.end_doy.is_none()
    }
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

#[skip_serializing_none]
#[serde_with::apply( Deferred => #[serde(skip_serializing_if = "Deferred::is_unloaded")])]
#[derive(Debug, Clone, toasty::Model, Serialize)]
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
    #[serde(skip_serializing_if = "RegionalHarvestWindow::is_empty")]
    pub harvest_window: RegionalHarvestWindow,
    #[index]
    pub native_plant_community_id: Option<u64>,
    #[belongs_to(key=native_plant_community_id, references=id)]
    pub native_plant_community: Deferred<NativePlantCommunity>,
}

#[skip_serializing_none]
#[serde_with::apply( Deferred => #[serde(skip_serializing_if = "Deferred::is_unloaded")])]
#[derive(Debug, Clone, toasty::Model, Serialize)]
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
        ConservationStatus, Origin, Region, RegionalTaxonStatus, WetlandIndicator,
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
                geometry: value.geometry.as_ref().map(|val| val.0.clone()),
                taxa,
                notes: value.notes.clone(),
            }
        }
    }
}
