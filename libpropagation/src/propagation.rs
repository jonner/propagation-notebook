use crate::{dto::ObjectReference, taxonomy::Taxon};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use toasty::Deferred;

#[derive(
    Debug, Clone, Copy, toasty::Embed, clap::ValueEnum, strum::Display, Deserialize, Serialize,
)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ProtocolType {
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
#[skip_serializing_none]
#[serde_with::apply( Deferred => #[serde(skip_serializing_if = "Deferred::is_unloaded")])]
#[derive(Debug, Clone, toasty::Model, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Protocol {
    #[key]
    #[auto]
    pub id: u64,

    #[index]
    pub name: String,

    pub instructions: String,
    pub notes: Option<String>,
    pub r#type: ProtocolType,

    #[serde(skip)]
    #[has_many]
    pub taxon_protocols: Deferred<Vec<TaxonProtocol>>,
}

impl From<Protocol> for ObjectReference {
    fn from(value: Protocol) -> Self {
        Self {
            id: value.id,
            name: Some(value.name),
        }
    }
}

impl From<&Protocol> for ObjectReference {
    fn from(value: &Protocol) -> Self {
        value.clone().into()
    }
}

// FIXME: implement Display instead?
impl Protocol {
    pub fn reference(&self) -> String {
        format!("{}: {}", self.id, self.name)
    }
}

// TODO: if protocols become parametrized, we'd need to add the parameters to
// this model...
#[skip_serializing_none]
#[serde_with::apply( Deferred => #[serde(skip_serializing_if = "Deferred::is_unloaded")])]
#[derive(Debug, Clone, toasty::Model, Serialize)]
pub struct TaxonProtocol {
    #[key]
    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: Deferred<Taxon>,

    #[key]
    #[index]
    pub protocol_id: u64,
    #[belongs_to(key=protocol_id, references=id)]
    pub protocol: Deferred<Protocol>,

    pub confidence: Option<u8>,
    pub notes: Option<String>,
}
