use std::fmt::Display;

use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{dto::ObjectReference, region::dto::RegionalTaxonStatusDetailsNoTaxon};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TaxonNameRank {
    pub id: u64,
    pub name: String,
    pub rank: super::Rank,
}

impl From<&super::Taxon> for TaxonNameRank {
    fn from(taxon: &super::Taxon) -> Self {
        taxon.clone().into()
    }
}

impl From<super::Taxon> for TaxonNameRank {
    fn from(taxon: super::Taxon) -> Self {
        Self {
            id: taxon.id,
            name: taxon.complete_name,
            rank: taxon.rank,
        }
    }
}

impl Display for TaxonNameRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({})", self.id, self.name, self.rank)
    }
}

#[skip_serializing_none]
#[serde_with::apply( Vec => #[serde(skip_serializing_if = "Vec::is_empty")])]
#[derive(Debug, Clone, Serialize)]
pub struct TaxonDetails {
    pub id: u64,
    pub name: String,
    pub rank: super::Rank,
    pub parent: Option<ObjectReference>,
    pub children: Vec<TaxonNameRank>,
    pub common_names: Vec<String>,
    pub synonyms: Vec<String>,
    pub regions: Vec<RegionalTaxonStatusDetailsNoTaxon>,
    pub collecting_data: Option<CollectingDataNoTaxon>,
    pub seed_cleaning: Vec<TaxonCleaningProcedureNoTaxon>,
    pub propagation_protocols: Vec<TaxonProtocolNoTaxon>,
    pub notes: Vec<TaxonNoteNoTaxon>,
}

impl From<super::Taxon> for TaxonDetails {
    fn from(value: super::Taxon) -> Self {
        Self {
            id: value.id,
            name: value.complete_name,
            rank: value.rank,
            parent: ObjectReference::from_deferred_option(value.parent, value.parent_id),
            children: match value.children.is_unloaded() {
                true => Vec::default(),
                false => value.children.get().iter().map(|t| t.into()).collect(),
            },
            common_names: match value.vernaculars.is_unloaded() {
                true => Vec::default(),
                false => value
                    .vernaculars
                    .get()
                    .iter()
                    .map(|v| v.name.clone())
                    .collect(),
            },
            synonyms: match value.synonyms.is_unloaded() {
                true => Vec::default(),
                false => value
                    .synonyms
                    .get()
                    .iter()
                    .map(|v| v.complete_name.clone())
                    .collect(),
            },
            regions: match value.regional_statuses.is_unloaded() {
                true => Vec::default(),
                false => value
                    .regional_statuses
                    .get()
                    .iter()
                    .map(|rs| rs.into())
                    .collect(),
            },
            collecting_data: match value.collecting_data.is_unloaded() {
                true => None,
                false => value.collecting_data.get().as_ref().map(|d| d.into()),
            },
            seed_cleaning: match value.cleaning_procedures.is_unloaded() {
                true => Vec::default(),
                false => value
                    .cleaning_procedures
                    .get()
                    .iter()
                    .map(Into::into)
                    .collect(),
            },
            propagation_protocols: match value.propagation_protocols.is_unloaded() {
                true => Vec::default(),
                false => value
                    .propagation_protocols
                    .get()
                    .iter()
                    .map(Into::into)
                    .collect(),
            },
            notes: match value.notes.is_unloaded() {
                true => Vec::default(),
                false => value.notes.get().iter().map(|n| n.into()).collect(),
            },
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TaxonNoteDetails {
    pub taxon: ObjectReference,
    #[serde(flatten)]
    pub core: TaxonNoteNoTaxon,
}

impl From<super::TaxonNote> for TaxonNoteDetails {
    fn from(value: super::TaxonNote) -> Self {
        Self {
            taxon: ObjectReference::from_deferred(&value.taxon, value.taxon_id),
            core: TaxonNoteNoTaxon {
                id: value.id,
                text: value.text,
                created_at: value.created_at,
                updated_at: value.updated_at,
            },
        }
    }
}

impl From<&super::TaxonNote> for TaxonNoteDetails {
    fn from(value: &super::TaxonNote) -> Self {
        value.clone().into()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TaxonNoteNoTaxon {
    pub id: u64,
    pub text: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}

impl From<super::TaxonNote> for TaxonNoteNoTaxon {
    fn from(value: super::TaxonNote) -> Self {
        Self {
            id: value.id,
            text: value.text,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<&super::TaxonNote> for TaxonNoteNoTaxon {
    fn from(value: &super::TaxonNote) -> Self {
        value.clone().into()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct CollectingDataDetails {
    pub taxon: ObjectReference,
    pub ripening_indicators: Option<String>,
    pub harvesting_notes: Option<String>,
    pub storage: Option<String>,
    pub storage_life: Option<String>,
}

impl From<&super::CollectingData> for CollectingDataDetails {
    fn from(value: &super::CollectingData) -> Self {
        value.clone().into()
    }
}
impl From<super::CollectingData> for CollectingDataDetails {
    fn from(value: super::CollectingData) -> Self {
        Self {
            taxon: ObjectReference::from_deferred(&value.taxon, value.taxon_id),
            ripening_indicators: value.ripening_indicators,
            harvesting_notes: value.harvesting_notes,
            storage: value.storage,
            storage_life: value.storage_life,
        }
    }
}
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct CollectingDataNoTaxon {
    pub ripening_indicators: Option<String>,
    pub harvesting_notes: Option<String>,
    pub storage: Option<String>,
    pub storage_life: Option<String>,
}

impl From<&super::CollectingData> for CollectingDataNoTaxon {
    fn from(value: &super::CollectingData) -> Self {
        value.clone().into()
    }
}
impl From<super::CollectingData> for CollectingDataNoTaxon {
    fn from(value: super::CollectingData) -> Self {
        Self {
            ripening_indicators: value.ripening_indicators,
            harvesting_notes: value.harvesting_notes,
            storage: value.storage,
            storage_life: value.storage_life,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TaxonCleaningProcedureDetails {
    pub taxon: ObjectReference,
    pub procedure: ObjectReference,
    pub notes: Option<String>,
}
impl From<super::TaxonCleaningProcedure> for TaxonCleaningProcedureDetails {
    fn from(value: super::TaxonCleaningProcedure) -> Self {
        Self {
            taxon: ObjectReference::from_deferred(&value.taxon, value.taxon_id),
            procedure: ObjectReference::from_deferred(&value.procedure, value.procedure_id),
            notes: value.notes,
        }
    }
}

impl From<&super::TaxonCleaningProcedure> for TaxonCleaningProcedureDetails {
    fn from(value: &super::TaxonCleaningProcedure) -> Self {
        value.clone().into()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TaxonCleaningProcedureNoTaxon {
    pub procedure: ObjectReference,
    pub notes: Option<String>,
}

impl From<super::TaxonCleaningProcedure> for TaxonCleaningProcedureNoTaxon {
    fn from(value: super::TaxonCleaningProcedure) -> Self {
        Self {
            procedure: ObjectReference::from_deferred(&value.procedure, value.procedure_id),
            notes: value.notes,
        }
    }
}

impl From<&super::TaxonCleaningProcedure> for TaxonCleaningProcedureNoTaxon {
    fn from(value: &super::TaxonCleaningProcedure) -> Self {
        value.clone().into()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TaxonProtocolDetails {
    pub taxon: ObjectReference,
    pub core: TaxonProtocolNoTaxon,
}

impl From<super::TaxonProtocol> for TaxonProtocolDetails {
    fn from(value: super::TaxonProtocol) -> Self {
        Self {
            taxon: ObjectReference::from_deferred(&value.taxon, value.taxon_id),
            core: TaxonProtocolNoTaxon {
                protocol: ObjectReference::from_deferred(&value.protocol, value.protocol_id),
                confidence: value.confidence,
                notes: value.notes,
            },
        }
    }
}

impl From<&super::TaxonProtocol> for TaxonProtocolDetails {
    fn from(value: &super::TaxonProtocol) -> Self {
        value.clone().into()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TaxonProtocolNoTaxon {
    pub protocol: ObjectReference,
    pub confidence: Option<u8>,
    pub notes: Option<String>,
}
impl From<super::TaxonProtocol> for TaxonProtocolNoTaxon {
    fn from(value: super::TaxonProtocol) -> Self {
        Self {
            protocol: ObjectReference::from_deferred(&value.protocol, value.protocol_id),
            confidence: value.confidence,
            notes: value.notes,
        }
    }
}

impl From<&super::TaxonProtocol> for TaxonProtocolNoTaxon {
    fn from(value: &super::TaxonProtocol) -> Self {
        value.clone().into()
    }
}
