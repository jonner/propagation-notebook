use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::dto::ObjectReference;

#[skip_serializing_none]
#[derive(Serialize, Clone, Debug)]
pub struct CitationCompact {
    pub id: u64,
    pub title: String,
    pub author: String,
    pub url: Option<String>,
}

impl From<super::Citation> for CitationCompact {
    fn from(value: super::Citation) -> Self {
        Self {
            id: value.id,
            title: value.title,
            author: value.author,
            url: value.url,
        }
    }
}

impl From<&super::Citation> for CitationCompact {
    fn from(value: &super::Citation) -> Self {
        value.clone().into()
    }
}

#[skip_serializing_none]
#[derive(Serialize, Clone, Debug)]
pub struct CitationDetails {
    pub id: u64,
    pub title: String,
    pub url: Option<String>,
    pub author: String,
    pub access_date: Option<jiff::civil::Date>,
    pub publication_year: Option<i16>,
    pub container_title: Option<String>,
    pub doi: Option<String>,

    pub cleaning_procedures: Vec<u64>,
    pub propagation_procedures: Vec<u64>,
    pub taxon_propagation_procedures: Vec<(u64, u64)>,
    pub taxon_notes: Vec<u64>,
}

impl From<super::Citation> for CitationDetails {
    fn from(value: super::Citation) -> Self {
        Self {
            id: value.id,
            title: value.title,
            url: value.url,
            author: value.author,
            access_date: value.access_date,
            publication_year: value.publication_year,
            container_title: value.container_title,
            doi: value.doi,
            cleaning_procedures: if value.cleaning_procedures.is_unloaded() {
                Vec::default()
            } else {
                value
                    .cleaning_procedures
                    .get()
                    .iter()
                    .map(|cpc| cpc.cleaning_id)
                    .collect()
            },
            propagation_procedures: if value.propagation_procedures.is_unloaded() {
                Vec::default()
            } else {
                value
                    .propagation_procedures
                    .get()
                    .iter()
                    .map(|ppc| ppc.propagation_id)
                    .collect()
            },
            taxon_propagation_procedures: if value.taxon_propagation_procedures.is_unloaded() {
                Vec::default()
            } else {
                value
                    .taxon_propagation_procedures
                    .get()
                    .iter()
                    .map(|tppc| (tppc.taxon_id, tppc.propagation_id))
                    .collect()
            },
            taxon_notes: if value.taxon_notes.is_unloaded() {
                Vec::default()
            } else {
                value
                    .taxon_notes
                    .get()
                    .iter()
                    .map(|nc| nc.note_id)
                    .collect()
            },
        }
    }
}

impl From<&super::Citation> for CitationDetails {
    fn from(value: &super::Citation) -> Self {
        value.clone().into()
    }
}

impl From<&super::Citation> for ObjectReference {
    fn from(value: &super::Citation) -> Self {
        Self {
            id: value.id,
            name: Some(value.title.clone()),
        }
    }
}
