use crate::taxonomy::Taxon;
use tabled::{Tabled, derive::display};
use toasty::{BelongsTo, HasMany};

#[derive(Debug, Clone, Copy, toasty::Embed, clap::ValueEnum, strum::Display)]
pub enum LightRequirement {
    #[column(variant = 1)]
    LightRequired,
    #[column(variant = 2)]
    DarkRequired,
    #[column(variant = 3)]
    NoPreference,
}

#[derive(Debug, Clone, toasty::Model, Tabled)]
#[tabled(display(Option, "display::option", "-"))]
pub struct ProtocolStep {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub protocol_id: u64,
    #[belongs_to(key=protocol_id, references=id)]
    #[tabled(skip)]
    pub protocol: BelongsTo<Protocol>,

    pub order: u64,
    pub step_type: ProtocolStepType,
    pub title: String,
    pub instructions: Option<String>,
    pub duration: Option<u64>,
    pub min_temp: Option<f32>,
    pub max_temp: Option<f32>,
    pub light: Option<LightRequirement>,
    pub moisture: Option<String>,
    pub materials: Option<String>,
    pub is_optional: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, toasty::Embed, clap::ValueEnum, strum::Display)]
pub enum ProtocolStepType {
    #[column(variant = 1)]
    Stratification,
    #[column(variant = 2)]
    Scarification,
    #[column(variant = 3)]
    Soaking,
    #[column(variant = 4)]
    Sowing,
    #[column(variant = 5)]
    Germination,
    #[column(variant = 6)]
    Transplanting,
    // #[column(variant = 7)]
    // CuttingPreparation,
    // #[column(variant = 8)]
    // Rooting,
    #[column(variant = 99)]
    Other,
}

#[derive(Debug, Clone, Copy, toasty::Embed, clap::ValueEnum, strum::Display)]
pub enum ProtocolType {
    Pretreatment,
    Germination,
    Establishment,
    // Propagation,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Protocol {
    #[key]
    #[auto]
    pub id: u64,
    pub name: String,
    pub notes: Option<String>,
    pub r#type: ProtocolType,

    #[has_many]
    pub steps: HasMany<ProtocolStep>,
    #[has_many]
    pub citations: HasMany<ProtocolCitation>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct TaxonProtocol {
    #[key]
    id: u64,
    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    taxon: BelongsTo<Taxon>,

    #[index]
    pub pretreatment_protocol_id: Option<u64>,
    #[belongs_to(key=pretreatment_protocol_id, references=id)]
    pub pretreatment: BelongsTo<Protocol>,
    #[index]
    pub germination_protocol_id: Option<u64>,
    #[belongs_to(key=germination_protocol_id, references=id)]
    pub germination: BelongsTo<Protocol>,
    #[index]
    pub establishment_protocol_id: Option<u64>,
    #[belongs_to(key=establishment_protocol_id, references=id)]
    pub establishment: BelongsTo<Protocol>,

    pub confidence: Option<u8>,
    pub success_rate: Option<f32>,
    pub notes: Option<String>,

    #[has_many]
    pub citations: HasMany<TaxonProtocolCitation>,
}

#[derive(Debug, Clone, Copy, toasty::Embed, clap::ValueEnum)]
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
    protocol: BelongsTo<Protocol>,

    #[key]
    citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    citation: BelongsTo<Citation>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct TaxonProtocolCitation {
    #[key]
    id: u64,

    #[index]
    taxon_protocol_id: u64,
    #[belongs_to(key=taxon_protocol_id, references=id)]
    taxon_protocol: BelongsTo<TaxonProtocol>,

    #[key]
    citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    citation: BelongsTo<Citation>,
}
