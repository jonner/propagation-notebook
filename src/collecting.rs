use tabled::{Tabled, derive::display};
use toasty::{BelongsTo, HasMany};

use crate::taxonomy::Taxon;

#[derive(Debug, Clone, toasty::Model)]
pub struct CollectingData {
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

// pivot table for associating a cleaning procedure with a taxon
#[derive(Debug, Clone, toasty::Model)]
pub struct TaxonCleaningProcedure {
    #[key]
    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: BelongsTo<Taxon>,

    // notes for customizing the procedure for this taxon
    pub notes: Option<String>,

    #[key]
    #[index]
    pub procedure_id: u64,
    #[belongs_to(key=procedure_id, references=id)]
    pub procedure: BelongsTo<CleaningProcedure>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct CleaningProcedure {
    #[auto]
    #[key]
    pub id: u64,
    pub name: String,
    pub notes: Option<String>,

    #[has_many(pair=procedure)]
    pub steps: HasMany<CleaningProcedureStep>,
    #[has_many(pair=procedure)]
    pub taxon_links: HasMany<TaxonCleaningProcedure>,
}

#[derive(Debug, Clone, Copy, toasty::Embed, strum::Display, clap::ValueEnum)]
pub enum OperationType {
    #[column(variant = 1)]
    Rub, // remove fuzz?
    #[column(variant = 2)]
    Screen,
    #[column(variant = 3)]
    Air,
    #[column(variant = 4)]
    Other,
}

#[derive(Debug, Clone, toasty::Model, Tabled)]
#[tabled(rename_all = "CamelCase")]
pub struct CleaningProcedureStep {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub procedure_id: u64,
    #[belongs_to(key=procedure_id, references=id)]
    #[tabled(skip)]
    pub procedure: BelongsTo<CleaningProcedure>,

    pub order: u64,
    pub operation_type: OperationType,
    #[tabled(display("display::option", "-"))]
    pub equipment: Option<String>,
    pub notes: String, // Description of the step
}

impl CleaningProcedureStep {
    pub fn summary(&self) -> String {
        format!(
            "{} // {} // {}",
            self.operation_type,
            self.notes,
            self.equipment.as_deref().unwrap_or("-"),
        )
    }
}
