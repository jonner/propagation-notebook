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
    pub citations: Deferred<Vec<ProtocolCitation>>,

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

    #[has_many]
    pub citations: Deferred<Vec<TaxonProtocolCitation>>,
}

#[derive(Debug, Clone, Copy, toasty::Embed, clap::ValueEnum, Deserialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum CitationType {
    #[column(variant = 1)]
    PeerReviewedPaper,
    #[column(variant = 2)]
    Book,
    #[column(variant = 3)]
    VendorCatalog,
    #[column(variant = 4)]
    ExpertInterview,
    #[column(variant = 5)]
    GovernmentDatabase,
    #[column(variant = 99)]
    Other,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Citation {
    #[auto]
    #[key]
    pub id: u64,

    pub r#type: CitationType,
    pub title: String,
    pub author: String,
    pub author_organization: Option<String>,
    pub publication_year: Option<u16>,
    pub url_doi: Option<String>,
    pub reliability: Option<u8>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ProtocolCitation {
    #[key]
    #[index]
    protocol_id: u64,
    #[belongs_to(key=protocol_id, references=id)]
    protocol: Deferred<Protocol>,

    #[key]
    citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    citation: Deferred<Citation>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct TaxonProtocolCitation {
    #[key]
    id: u64,

    #[index]
    taxon_protocol_id: u64,
    #[belongs_to(key=taxon_protocol_id, references=id)]
    taxon_protocol: Deferred<TaxonProtocol>,

    #[key]
    citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    citation: Deferred<Citation>,
}
