use toasty::{BelongsTo, HasMany};

use crate::taxonomy::Taxon;

#[derive(Debug, Clone, Copy, toasty::Embed, strum::Display, clap::ValueEnum)]
pub enum WetlandIndicator {
    #[column(variant = 1)]
    ObligateWetland,
    #[column(variant = 2)]
    FacultativeWetland,
    #[column(variant = 3)]
    Facultative,
    #[column(variant = 4)]
    FacultativeUpland,
    #[column(variant = 5)]
    Upland,
    #[column(variant = 99)]
    Other,
}

#[derive(Debug, Clone, Copy, toasty::Embed, strum::Display, clap::ValueEnum)]
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
    pub bounds: Option<String>,

    #[has_many]
    pub taxon_statuses: HasMany<RegionalTaxonStatus>,
    #[has_many]
    pub npcs: HasMany<NativePlantCommunity>,
}

#[derive(Debug, Clone, Copy, toasty::Embed, strum::Display, clap::ValueEnum)]
pub enum NativeStatus {
    #[column(variant = 1)]
    Native,
    #[column(variant = 2)]
    Introduced,
    #[column(variant = 3)]
    Unknown,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct RegionalTaxonStatus {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: BelongsTo<Taxon>,

    #[index]
    pub region_id: u64,
    #[belongs_to(key=region_id, references=id)]
    pub region: BelongsTo<Region>,

    pub native_status: Option<NativeStatus>,
    // generally 0-10?
    pub c_value: Option<u64>,
    pub conservation_status: Option<ConservationStatus>,
    pub wetland_indicator: Option<WetlandIndicator>,
    // harvest phenology
    pub window_start: Option<jiff::civil::Date>,
    pub window_end: Option<jiff::civil::Date>,

    #[index]
    pub native_plant_community_id: Option<u64>,
    #[belongs_to(key=native_plant_community_id, references=id)]
    pub native_plant_community: BelongsTo<NativePlantCommunity>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct NativePlantCommunity {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub region_id: u64,
    #[belongs_to(key=region_id, references=id)]
    pub region: BelongsTo<Region>,

    #[index]
    pub name: String,

    #[has_many]
    regional_taxon_statuses: HasMany<RegionalTaxonStatus>,
}
