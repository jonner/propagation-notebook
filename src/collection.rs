#[derive(Debug, toasty::Model)]
pub struct CollectionData {
    #[auto]
    #[key]
    id: u64,
    taxon_id: u64,
    ripening_indicators: String,
    storage: Option<String>,
}

#[derive(Debug, toasty::Model)]
pub struct CleaningProcedure {
    #[auto]
    #[key]
    id: u64,
    taxon_id: u64,
    notes: Option<String>,
}

#[derive(Debug, toasty::Embed)]
pub enum CleaningType {
    #[column(variant = 1)]
    Rub, // remove fuzz?
    #[column(variant = 2)]
    Screen,
    #[column(variant = 3)]
    Air,
    #[column(variant = 4)]
    Other,
}

#[derive(Debug, toasty::Model)]
pub struct CleaningProcedureStep {
    #[auto]
    #[key]
    id: u64,
    procedure_id: u64,
    order: u64,
    cleaning_type: CleaningType,
    equipment: String,
    notes: Option<String>, // e.g screen size
}

// Add a way to import phenology information from inaturalist?
#[derive(Debug, toasty::Model)]
pub struct Phenology {
    #[auto]
    #[key]
    id: u64,
    taxon_id: u64,
    region_id: u64,
    window_start: jiff::civil::Date,
    window_end: jiff::civil::Date,
}
