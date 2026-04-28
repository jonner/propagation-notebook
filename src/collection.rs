use toasty::{BelongsTo, HasMany};

use crate::{region::Region, taxonomy::Taxon};

#[derive(Debug, toasty::Model)]
pub struct CollectionData {
    #[auto]
    #[key]
    id: u64,

    #[index]
    taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    taxon: BelongsTo<Taxon>,

    ripening_indicators: String,
    storage: Option<String>,
}

#[derive(Debug, toasty::Model)]
pub struct CleaningProcedure {
    #[auto]
    #[key]
    id: u64,

    #[index]
    taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    taxon: BelongsTo<Taxon>,

    notes: Option<String>,

    #[has_many(pair=procedure)]
    steps: HasMany<CleaningProcedureStep>,
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

    #[index]
    procedure_id: u64,
    #[belongs_to(key=procedure_id, references=id)]
    procedure: BelongsTo<CleaningProcedure>,

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

    #[index]
    taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    taxon: BelongsTo<Taxon>,

    #[index]
    region_id: u64,
    #[belongs_to(key=region_id, references=id)]
    region: BelongsTo<Region>,

    window_start: jiff::civil::Date,
    window_end: jiff::civil::Date,
}
