#[derive(Debug, thiserror::Error)]
pub enum ImportExportError {
    #[error("Database already contains {0} taxa. Refusing to import.")]
    TaxonomyPresent(u64),
    #[error("A region with the name '{0}' already exists")]
    RegionExists(String),
    #[error("Unable to find a taxon equivalent to '{0}' in the database")]
    NoMatchingTaxon(String),
    #[error(transparent)]
    Database(#[from] toasty::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    FileFormat(#[from] serde_yaml::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Runtime {
    #[error("Unable to determine project data directory")]
    ProjectDirNotFound,
    #[error("Invalid environment variable: {0}")]
    InvalidEnvVar(String),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Db(#[from] toasty::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Runtime(Runtime),
}
