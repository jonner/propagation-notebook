use toasty::{BelongsTo, HasMany};

use crate::taxonomy::Taxon;

#[derive(Debug, Clone, Copy, toasty::Embed)]
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

#[derive(Debug, Clone, Copy, toasty::Embed)]
pub enum PropagationType {
    #[column(variant = 1)]
    Sexual,
    #[column(variant = 2)]
    Asexual,
}

#[derive(Debug, Clone, Copy, toasty::Embed)]
pub enum DifficultyLevel {
    #[column(variant = 1)]
    Easy,
    #[column(variant = 2)]
    Moderate,
    #[column(variant = 3)]
    Challenging,
    #[column(variant = 4)]
    Expert,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Procedure {
    #[auto]
    #[key]
    pub id: u64,

    pub propagation_type: PropagationType,
    pub difficulty: DifficultyLevel,
    // link to an external table of notes?
    pub notes: Option<String>,

    #[has_many]
    pub sexual_steps: HasMany<SexualMethodStep>,
    #[has_many]
    pub asexual_steps: HasMany<AsexualMethodStep>,
    #[has_many]
    pub protocols: HasMany<Protocol>,
}

#[derive(Debug, Clone, Copy, toasty::Embed)]
pub enum TreatmentType {
    #[column(variant = 1)]
    ColdMoistStratification,
    #[column(variant = 2)]
    WarmStratification,
    #[column(variant = 3)]
    MechanicalScarification,
    #[column(variant = 4)]
    ChemicalScarification,
    #[column(variant = 5)]
    SmokeWater,
    #[column(variant = 6)]
    PreSoak,
}

#[derive(Debug, Clone, Copy, toasty::Embed)]
pub enum LightRequirement {
    #[column(variant = 1)]
    LightRequired,
    #[column(variant = 2)]
    DarkRequired,
    #[column(variant = 3)]
    NoPreference,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct SexualMethodStep {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub procedure_id: u64,
    #[belongs_to(key=procedure_id, references=id)]
    pub procedure: BelongsTo<Procedure>,

    pub step_order: u64,
    pub treatment_type: TreatmentType,
    pub duration_days: u64,
    pub temp_day: u64,
    pub temp_night: u64,
    pub light_requirements: LightRequirement,
}

#[derive(Debug, Clone, Copy, toasty::Embed)]
pub enum AsexualMethodType {
    #[column(variant = 1)]
    RhizomeDivision,
    #[column(variant = 2)]
    StemCutting,
    #[column(variant = 3)]
    RootCutting,
    #[column(variant = 4)]
    TissueCulture,
    #[column(variant = 5)]
    Layering,
    #[column(variant = 99)]
    Other,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct AsexualMethodStep {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub procedure_id: u64,
    #[belongs_to(key=procedure_id, references=id)]
    pub procedure: BelongsTo<Procedure>,

    pub method_type: AsexualMethodType,
    pub hormone_treatment: Option<String>,
    pub substrate_media: Option<String>,
    pub moise_humidity_requirement: Option<String>,
    pub optimal_timing: Option<String>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct CultureEnvironment {
    #[auto]
    #[key]
    pub id: u64,

    // mm
    pub sowing_depth: u64,
    pub depth_description: Option<String>, // (Enum/Text: e.g., "Surface Sown," "1x Seed Diameter," "1/4 inch")
    pub media_type: Option<String>,        // enum?
    pub compaction_level: Option<String>,  // (Enum: Lightly pressed, Firmly packed, Loose)
    pub moisture_regime: Option<String>, // (Enum: Saturated/Wet, Consistently Moist, Dry-out between waterings)
    pub container_type: Option<String>, // (Enum: Plug tray, Deep conetainer, Open flat, Soil block)
    pub is_experimental: bool,
    // link to an external table of notes?
    pub notes: Option<String>,

    #[has_many(pair=environment)]
    pub procedures: HasMany<Protocol>,
}

// a combination of seed prep and sowing
#[derive(Debug, Clone, toasty::Model)]
pub struct Protocol {
    #[auto]
    #[key]
    pub id: u64,

    pub name: String,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: BelongsTo<Taxon>,

    #[index]
    pub protocol_id: u64,
    #[belongs_to(key=protocol_id, references=id)]
    pub procedure: BelongsTo<Procedure>,

    #[index]
    pub environment_id: u64,
    #[belongs_to(key=environment_id, references=id)]
    pub environment: BelongsTo<CultureEnvironment>,

    #[index]
    pub citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    pub citation: BelongsTo<Citation>,

    pub notes: Option<String>,
}
