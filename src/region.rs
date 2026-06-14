use std::fmt::Display;

use serde::{Deserialize, Serialize};
use toasty::Deferred;

use crate::taxonomy::Taxon;

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

impl Region {
    pub fn reference(&self) -> String {
        format!("{}: {}", self.id, self.name)
    }
}

#[derive(
    Debug, Clone, Copy, toasty::Embed, strum::Display, clap::ValueEnum, Serialize, Deserialize,
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
    pub start: Option<jiff::civil::Date>,
    pub end: Option<jiff::civil::Date>,
}

impl Display for RegionalHarvestWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            self.start
                .map(|d| d.strftime("%b %d").to_string())
                .unwrap_or("?".to_string()),
            self.end
                .map(|d| d.strftime("%b %d").to_string())
                .unwrap_or("?".to_string())
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
