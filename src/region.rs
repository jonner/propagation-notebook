use toasty::{BelongsTo, HasMany};

use crate::taxonomy::Taxon;

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
    id: u64,

    #[index]
    name: String,
    // FIXME: geojson??
    bound: String,

    #[has_many]
    statuses: HasMany<RegionStatus>,
    #[has_many]
    npcs: HasMany<NativePlantCommunity>,
}

#[derive(Debug, toasty::Model)]
pub struct RegionStatus {
    #[auto]
    #[key]
    id: u64,

    #[index]
    taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    taxon: BelongsTo<Taxon>,

    #[index]
    region_id: u64,
    #[belongs_to(key=region_id, references=id)]
    region: BelongsTo<Region>,

    // generally 0-10?
    c_value: Option<u64>,
    conservation_status: ConservationStatus,
    wetland_indicator: WetlandIndicator,
    native_plant_community_id: u64,
}

#[derive(Debug, toasty::Model)]
pub struct NativePlantCommunity {
    #[auto]
    #[key]
    id: u64,

    #[index]
    region_id: u64,
    #[belongs_to(key=region_id, references=id)]
    region: BelongsTo<Region>,

    #[index]
    name: String,
}
