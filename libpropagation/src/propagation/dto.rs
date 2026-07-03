use serde::Serialize;

use crate::dto::ObjectReference;

#[derive(Debug, Clone, Serialize)]
pub struct PropagationProcedureDetails {
    pub id: u64,
    pub name: String,
    pub r#type: super::ProcedureType,
    pub instructions: String,
    pub notes: Option<String>,
    pub citation: Option<String>,
    pub taxa: Vec<ObjectReference>,
}

impl From<super::PropagationProcedure> for PropagationProcedureDetails {
    fn from(value: super::PropagationProcedure) -> Self {
        Self {
            id: value.id,
            name: value.name,
            r#type: value.r#type,
            instructions: value.instructions,
            notes: value.notes,
            citation: value.citation,
            taxa: match value.taxa.is_unloaded() {
                true => Vec::default(),
                false => value
                    .taxa
                    .get()
                    .iter()
                    .map(|tp| ObjectReference::from_deferred(&tp.taxon, tp.taxon_id))
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PropagationProcedureCompact {
    pub id: u64,
    pub name: String,
    pub r#type: super::ProcedureType,
}

impl From<super::PropagationProcedure> for PropagationProcedureCompact {
    fn from(value: super::PropagationProcedure) -> Self {
        Self {
            id: value.id,
            name: value.name,
            r#type: value.r#type,
        }
    }
}
