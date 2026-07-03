use toasty::Deferred;

use crate::{
    citation::{Citation, CleaningProcedureCitation, TaxonCleaningProcedureCitation},
    dto::ObjectReference,
    taxonomy::Taxon,
};

pub mod dto;

#[derive(Debug, Clone, toasty::Model)]
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
#[derive(Debug, Clone, toasty::Model)]
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
    #[has_many(pair=taxon_cleaning)]
    pub citation_links: Deferred<Vec<TaxonCleaningProcedureCitation>>,
    #[has_many(via=citation_links.citation)]
    pub citations: Deferred<Vec<Citation>>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct CleaningProcedure {
    #[auto]
    #[key]
    pub id: u64,
    pub name: String,
    pub notes: Option<String>,
    pub instructions: String,
    #[has_many(pair=procedure)]
    pub taxon_links: Deferred<Vec<TaxonCleaningProcedure>>,
    #[has_many(pair=cleaning)]
    pub citation_links: Deferred<Vec<CleaningProcedureCitation>>,
    #[has_many(via=citation_links.citation)]
    pub citations: Deferred<Vec<Citation>>,
}

impl From<CleaningProcedure> for ObjectReference {
    fn from(value: CleaningProcedure) -> Self {
        Self {
            id: value.id,
            name: Some(value.name),
        }
    }
}

impl From<&CleaningProcedure> for ObjectReference {
    fn from(value: &CleaningProcedure) -> Self {
        Self {
            id: value.id,
            name: Some(value.name.clone()),
        }
    }
}

impl CleaningProcedure {
    pub fn reference(&self) -> ObjectReference {
        self.into()
    }
}
