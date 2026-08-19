use std::fmt::Display;

use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    collecting::dto::CleaningProcedureCompact, dto::ObjectReference,
    region::dto::RegionalTaxonStatusDetailsNoTaxon,
};

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
    pub seed_cleaning: Vec<CleaningProcedureCompact>,
    pub propagation_procedures: Vec<TaxonPropagationProcedureCompact>,
    pub notes: Vec<TaxonNoteNoTaxon>,
    pub itis_id: u64,
    pub inaturalist_id: Option<u64>,
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
            propagation_procedures: match value.propagation_procedures.is_unloaded() {
                true => Vec::default(),
                false => value
                    .propagation_procedures
                    .get()
                    .iter()
                    .map(Into::into)
                    .collect(),
            },
            notes: match value.notes.is_unloaded() {
                true => Vec::default(),
                false => value.notes.get().iter().map(|n| n.into()).collect(),
            },
            itis_id: value.itis_id,
            inaturalist_id: value.inaturalist_id,
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
            core: TaxonNoteNoTaxon::from(value),
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
pub struct TaxonPropagationProcedureDetails {
    pub taxon: ObjectReference,
    pub core: TaxonPropagationProcedureCompact,
    pub citations: Vec<ObjectReference>,
}

impl From<super::TaxonPropagationProcedure> for TaxonPropagationProcedureDetails {
    fn from(value: super::TaxonPropagationProcedure) -> Self {
        let citations = if value.citation_links.is_unloaded() {
            &Default::default()
        } else {
            value.citation_links.get()
        };
        Self {
            taxon: ObjectReference::from_deferred(&value.taxon, value.taxon_id),
            citations: citations.iter().map(|v| v.citation.get().into()).collect(),
            core: TaxonPropagationProcedureCompact::from(value),
        }
    }
}

impl From<&super::TaxonPropagationProcedure> for TaxonPropagationProcedureDetails {
    fn from(value: &super::TaxonPropagationProcedure) -> Self {
        value.clone().into()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct TaxonPropagationProcedureCompact {
    pub propagation: ObjectReference,
    pub confidence: Option<u8>,
    pub notes: Option<String>,
}
impl From<super::TaxonPropagationProcedure> for TaxonPropagationProcedureCompact {
    fn from(value: super::TaxonPropagationProcedure) -> Self {
        Self {
            propagation: ObjectReference::from_deferred(&value.propagation, value.propagation_id),
            confidence: value.confidence,
            notes: value.notes,
        }
    }
}

impl From<&super::TaxonPropagationProcedure> for TaxonPropagationProcedureCompact {
    fn from(value: &super::TaxonPropagationProcedure) -> Self {
        value.clone().into()
    }
}
