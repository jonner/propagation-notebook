use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::dto::ObjectReference;

#[skip_serializing_none]
#[serde_with::apply( Vec => #[serde(skip_serializing_if = "Vec::is_empty")])]
#[derive(Debug, Clone, Serialize)]
pub struct CleaningProcedureCompact {
    pub id: u64,
    pub name: String,
}

impl From<super::CleaningProcedure> for CleaningProcedureCompact {
    fn from(value: super::CleaningProcedure) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl From<&super::CleaningProcedure> for CleaningProcedureCompact {
    fn from(value: &super::CleaningProcedure) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
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
    pub citations: Vec<ObjectReference>,
}

impl From<super::CleaningProcedure> for CleaningProcedureDetails {
    fn from(value: super::CleaningProcedure) -> Self {
        Self {
            id: value.id,
            name: value.name,
            notes: value.notes,
            instructions: value.instructions,
            citations: match value.citations.is_unloaded() {
                true => Vec::default(),
                false => value
                    .citations
                    .get()
                    .iter()
                    .map(ObjectReference::from)
                    .collect(),
            },
        }
    }
}
