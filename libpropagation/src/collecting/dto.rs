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
    pub citation: Option<String>,
    pub taxa: Vec<ObjectReference>,
}

impl From<super::CleaningProcedure> for CleaningProcedureDetails {
    fn from(value: super::CleaningProcedure) -> Self {
        Self {
            id: value.id,
            name: value.name,
            notes: value.notes,
            instructions: value.instructions,
            citation: value.citation,
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
