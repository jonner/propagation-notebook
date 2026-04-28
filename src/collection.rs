use toasty::{BelongsTo, HasMany};

use crate::{region::Region, taxonomy::Taxon};

#[derive(Debug, toasty::Model)]
pub struct CollectionData {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: BelongsTo<Taxon>,

    pub ripening_indicators: String,
    pub storage: Option<String>,
}

#[derive(Debug, toasty::Model)]
pub struct CleaningProcedure {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: BelongsTo<Taxon>,

    pub notes: Option<String>,

    #[has_many(pair=procedure)]
    pub steps: HasMany<CleaningProcedureStep>,
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
    pub id: u64,

    #[index]
    pub procedure_id: u64,
    #[belongs_to(key=procedure_id, references=id)]
    pub procedure: BelongsTo<CleaningProcedure>,

    pub order: u64,
    pub cleaning_type: CleaningType,
    pub equipment: String,
    pub notes: Option<String>, // e.g screen size
}

// Add a way to import phenology information from inaturalist?
#[derive(Debug, toasty::Model)]
pub struct Phenology {
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

    pub window_start: jiff::civil::Date,
    pub window_end: jiff::civil::Date,
}
