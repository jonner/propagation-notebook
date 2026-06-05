use crate::taxonomy::Taxon;
use serde::Deserialize;
use toasty::Deferred;

#[derive(Debug, Clone, Copy, toasty::Embed, clap::ValueEnum, strum::Display, Deserialize)]
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
#[derive(Debug, Clone, toasty::Model, Deserialize)]
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

// TODO: if protocols become parametrized, we'd need to add the parameters to
// this model...
#[derive(Debug, Clone, toasty::Model)]
pub struct TaxonProtocol {
    #[key]
    id: u64,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    taxon: Deferred<Taxon>,

    #[index]
    pub protocol_id: Option<u64>,
    #[belongs_to(key=protocol_id, references=id)]
    pub protocol: Deferred<Protocol>,

    pub confidence: Option<u8>,
    pub notes: Option<String>,
}
