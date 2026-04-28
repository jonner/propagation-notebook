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
    name: u64,
    // FIXME: geojson??
    bound: String,
}

#[derive(Debug, toasty::Model)]
pub struct RegionStatus {
    #[auto]
    #[key]
    id: u64,
    taxon_id: u64,
    region_id: u64,
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
    region_id: u64,
    name: String,
}
