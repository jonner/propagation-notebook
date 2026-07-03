use crate::{dto::ObjectReference, taxonomy::TaxonPropagationProcedure};
use serde::{Deserialize, Serialize};
use toasty::Deferred;

pub mod dto;

#[derive(
    Debug, Clone, Copy, toasty::Embed, clap::ValueEnum, strum::Display, Deserialize, Serialize,
)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ProcedureType {
    #[column(variant = 1)]
    Pretreatment,
    #[column(variant = 2)]
    Germination,
    #[column(variant = 3)]
    Establishment,
    // #[column(variant = 4)]
    // Propagation,
    #[column(variant = 99)]
    Other,
}

// TODO: offer customizable parameters (e.g. 'days' for cold moist stratification)?
#[derive(Debug, Clone, toasty::Model)]
pub struct PropagationProcedure {
    #[key]
    #[auto]
    pub id: u64,

    #[index]
    pub name: String,

    pub instructions: String,
    pub notes: Option<String>,
    pub r#type: ProcedureType,
    pub citation: Option<String>,

    #[has_many(pair=propagation)]
    pub taxa: Deferred<Vec<TaxonPropagationProcedure>>,
}

impl From<PropagationProcedure> for ObjectReference {
    fn from(value: PropagationProcedure) -> Self {
        Self {
            id: value.id,
            name: Some(value.name),
        }
    }
}

impl From<&PropagationProcedure> for ObjectReference {
    fn from(value: &PropagationProcedure) -> Self {
        Self {
            id: value.id,
            name: Some(value.name.clone()),
        }
    }
}

impl PropagationProcedure {
    pub fn reference(&self) -> ObjectReference {
        self.into()
    }
}
