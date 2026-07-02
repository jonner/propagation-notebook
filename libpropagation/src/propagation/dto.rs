use serde::Serialize;

use crate::dto::ObjectReference;

#[derive(Debug, Clone, Serialize)]
pub struct ProtocolDetails {
    pub id: u64,
    pub name: String,
    pub r#type: super::ProtocolType,
    pub instructions: String,
    pub notes: Option<String>,
    pub taxa: Vec<ObjectReference>,
}

impl From<super::Protocol> for ProtocolDetails {
    fn from(value: super::Protocol) -> Self {
        Self {
            id: value.id,
            name: value.name,
            r#type: value.r#type,
            instructions: value.instructions.clone(),
            notes: value.notes.clone(),
            taxa: match value.taxon_protocols.is_unloaded() {
                true => Vec::default(),
                false => value
                    .taxon_protocols
                    .get()
                    .iter()
                    .map(|tp| ObjectReference::from_deferred(&tp.taxon, tp.taxon_id))
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProtocolCompact {
    pub id: u64,
    pub name: String,
    pub r#type: super::ProtocolType,
}

impl From<super::Protocol> for ProtocolCompact {
    fn from(value: super::Protocol) -> Self {
        Self {
            id: value.id,
            name: value.name,
            r#type: value.r#type,
        }
    }
}
