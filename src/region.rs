use toasty::{BelongsTo, HasMany};

use crate::{collection::Phenology, taxonomy::Taxon};

#[derive(Debug, toasty::Embed)]
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

#[derive(Debug, toasty::Embed)]
pub enum ConservationStatus {
    #[column(variant = 1)]
    Endangered,
    #[column(variant = 2)]
    Threatened,
    #[column(variant = 3)]
    SpecialConcern,
}

#[derive(Debug, toasty::Model)]
pub struct Region {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub name: String,
    // FIXME: geojson??
    pub bounds: String,

    #[has_many]
    pub statuses: HasMany<RegionStatus>,
    #[has_many]
    pub npcs: HasMany<NativePlantCommunity>,
    #[has_many]
    pub phenologies: HasMany<Phenology>,
}

#[derive(Debug, toasty::Model)]
pub struct RegionStatus {
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

    // generally 0-10?
    pub c_value: Option<u64>,
    pub conservation_status: ConservationStatus,
    pub wetland_indicator: WetlandIndicator,
    pub native_plant_community_id: u64,
}

#[derive(Debug, toasty::Model)]
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
}
