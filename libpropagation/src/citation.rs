use toasty::Deferred;

use crate::{
    collecting::{CleaningProcedure, TaxonCleaningProcedure},
    propagation::PropagationProcedure,
    taxonomy::TaxonPropagationProcedure,
};

#[derive(Debug, Clone, toasty::Model)]
pub struct Citation {
    #[key]
    #[auto]
    pub id: u64,
    pub text: String,
    pub url: Option<String>,
    pub author: Option<String>,

    #[has_many]
    pub cleaning_procedures: Deferred<Vec<CleaningProcedureCitation>>,
    #[has_many]
    pub taxon_cleaning_procedures: Deferred<Vec<TaxonCleaningProcedureCitation>>,
    #[has_many]
    pub propagation_procedures: Deferred<Vec<PropagationProcedureCitation>>,
    #[has_many]
    pub taxon_propagation_procedures: Deferred<Vec<TaxonPropagationProcedureCitation>>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct CleaningProcedureCitation {
    #[key]
    #[index]
    pub citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    pub citation: Deferred<Citation>,
    #[key]
    #[index]
    pub cleaning_id: u64,
    #[belongs_to(key=cleaning_id, references=id)]
    pub cleaning: Deferred<CleaningProcedure>,
}

#[derive(Debug, Clone, toasty::Model)]
#[index(taxon_id, procedure_id)]
pub struct TaxonCleaningProcedureCitation {
    #[key]
    #[index]
    pub citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    pub citation: Deferred<Citation>,
    #[key]
    pub taxon_id: u64,
    #[key]
    pub procedure_id: u64,
    #[belongs_to(key=[taxon_id, procedure_id], references=[taxon_id, procedure_id])]
    pub taxon_cleaning: Deferred<TaxonCleaningProcedure>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct PropagationProcedureCitation {
    #[key]
    #[index]
    pub citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    pub citation: Deferred<Citation>,
    #[key]
    #[index]
    pub propagation_id: u64,
    #[belongs_to(key=propagation_id, references=id)]
    pub propagation: Deferred<PropagationProcedure>,
}

#[derive(Debug, Clone, toasty::Model)]
#[index(taxon_id, propagation_id)]
pub struct TaxonPropagationProcedureCitation {
    #[key]
    #[index]
    pub citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    pub citation: Deferred<Citation>,
    #[key]
    pub propagation_id: u64,
    #[key]
    pub taxon_id: u64,
    #[belongs_to(key=[taxon_id, propagation_id], references=[taxon_id, propagation_id])]
    pub taxon_propagation: Deferred<TaxonPropagationProcedure>,
}
