use toasty::Deferred;

use crate::{dto::ObjectReference, taxonomy::Taxon};

pub mod dto {
    use serde::Serialize;
    use serde_with::skip_serializing_none;

    use crate::dto::ObjectReference;

    #[skip_serializing_none]
    #[serde_with::apply( Vec => #[serde(skip_serializing_if = "Vec::is_empty")])]
    #[derive(Debug, Clone, Serialize)]
    pub struct CleaningProcedureCompact {
        pub id: u64,
        pub name: String,
        pub n_taxa: Option<usize>,
    }

    impl From<super::CleaningProcedure> for CleaningProcedureCompact {
        fn from(value: super::CleaningProcedure) -> Self {
            Self {
                id: value.id,
                name: value.name,
                n_taxa: match value.taxon_links.is_unloaded() {
                    true => None,
                    false => Some(value.taxon_links.get().len()),
                },
            }
        }
    }

    #[skip_serializing_none]
    #[serde_with::apply( Vec => #[serde(skip_serializing_if = "Vec::is_empty")])]
    #[derive(Debug, Clone, Serialize)]
    pub struct CleaningProcedureDetails {
        pub id: u64,
        pub name: String,
        pub notes: Option<String>,
        pub instructions: String,
        pub taxa: Vec<ObjectReference>,
    }

    impl From<super::CleaningProcedure> for CleaningProcedureDetails {
        fn from(value: super::CleaningProcedure) -> Self {
            Self {
                id: value.id,
                name: value.name,
                notes: value.notes,
                instructions: value.instructions,
                taxa: match value.taxon_links.is_unloaded() {
                    true => Vec::default(),
                    false => value
                        .taxon_links
                        .get()
                        .iter()
                        .map(|tl| ObjectReference::from_deferred(&tl.taxon, tl.taxon_id))
                        .collect(),
                },
            }
        }
    }
}

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
