#[derive(Debug, toasty::Embed)]
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

#[derive(Debug, toasty::Model)]
pub struct Citations {
    #[auto]
    #[key]
    id: u64,
    r#type: CitationType,
    title: String,
    author: String,
    author_organization: Option<String>,
    publication_year: Option<u16>,
    url_doi: Option<String>,
    reliability: Option<u8>,
}

#[derive(Debug, toasty::Embed)]
pub enum PropagationType {
    #[column(variant = 1)]
    Sexual,
    #[column(variant = 2)]
    Asexual,
}

#[derive(Debug, toasty::Embed)]
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

#[derive(Debug, toasty::Model)]
pub struct Protocol {
    #[auto]
    #[key]
    id: u64,
    taxon_id: u64,
    citation_id: u64,
    propagation_type: PropagationType,
    difficulty: DifficultyLevel,
    // link to an external table of notes?
    notes: Option<String>,
}

#[derive(Debug, toasty::Embed)]
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

#[derive(Debug, toasty::Embed)]
pub enum LightRequirement {
    #[column(variant = 1)]
    LightRequired,
    #[column(variant = 2)]
    DarkRequired,
    #[column(variant = 3)]
    NoPreference,
}

#[derive(Debug, toasty::Model)]
pub struct SexualMethodStep {
    #[auto]
    #[key]
    id: u64,
    protocol_id: u64,
    step_order: u64,
    treatment_type: TreatmentType,
    duration_days: u64,
    temp_day: u64,
    temp_night: u64,
    light_requirements: LightRequirement,
}

#[derive(Debug, toasty::Embed)]
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

#[derive(Debug, toasty::Model)]
pub struct AsexualMethodStep {
    #[auto]
    #[key]
    id: u64,
    protocol_id: u64,
    method_type: AsexualMethodType,
    hormone_treatment: Option<String>,
    substrate_media: Option<String>,
    moise_humidity_requirement: Option<String>,
    optimal_timing: Option<String>,
}

#[derive(Debug, toasty::Model)]
pub struct CultureEnvironment {
    #[auto]
    #[key]
    id: u64,
    taxon_id: u64,
    // mm
    sowing_depth: u64,
    depth_description: Option<String>, // (Enum/Text: e.g., "Surface Sown," "1x Seed Diameter," "1/4 inch")
    media_type: Option<String>,        // enum?
    compaction_level: Option<String>,  // (Enum: Lightly pressed, Firmly packed, Loose)
    moisture_regime: Option<String>, // (Enum: Saturated/Wet, Consistently Moist, Dry-out between waterings)
    container_type: Option<String>,  // (Enum: Plug tray, Deep conetainer, Open flat, Soil block)
    is_experimental: bool,
    // link to an external table of notes?
    notes: Option<String>,
}

// a combination of seed prep and sowing
#[derive(Debug, toasty::Model)]
pub struct Procedure {
    #[auto]
    #[key]
    id: u64,
    protocol_id: u64,
    environment_id: u64,
    notes: Option<String>,
}
