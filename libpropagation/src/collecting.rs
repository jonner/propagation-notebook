use serde::Serialize;
use serde_with::skip_serializing_none;
use toasty::Deferred;

use crate::taxonomy::Taxon;

#[skip_serializing_none]
#[serde_with::apply( Deferred => #[serde(skip_serializing_if = "Deferred::is_unloaded")])]
#[derive(Debug, Clone, toasty::Model, Serialize)]
pub struct CollectingData {
    #[auto]
    #[key]
    pub id: u64,

    #[unique]
    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: Deferred<Taxon>,

    pub ripening_indicators: Option<String>,
    pub harvesting_notes: Option<String>,
    pub storage: Option<String>,
    pub storage_life: Option<String>,
}

// pivot table for associating a cleaning procedure with a taxon
#[skip_serializing_none]
#[serde_with::apply( Deferred => #[serde(skip_serializing_if = "Deferred::is_unloaded")])]
#[derive(Debug, Clone, toasty::Model, Serialize)]
pub struct TaxonCleaningProcedure {
    #[key]
    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: Deferred<Taxon>,

    // notes for customizing the procedure for this taxon
    pub notes: Option<String>,

    #[key]
    #[index]
    pub procedure_id: u64,
    #[belongs_to(key=procedure_id, references=id)]
    pub procedure: Deferred<CleaningProcedure>,
}

#[skip_serializing_none]
#[serde_with::apply( Deferred => #[serde(skip_serializing_if = "Deferred::is_unloaded")])]
#[derive(Debug, Clone, toasty::Model, Serialize)]
pub struct CleaningProcedure {
    #[auto]
    #[key]
    pub id: u64,
    pub name: String,
    pub notes: Option<String>,

    pub instructions: String,
    #[has_many(pair=procedure)]
    pub taxon_links: Deferred<Vec<TaxonCleaningProcedure>>,
}
