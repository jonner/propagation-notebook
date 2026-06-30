use directories::ProjectDirs;
use toasty::ModelSet;

pub mod collecting;
pub mod propagation;
pub mod region;
pub mod taxonomy;

pub fn models() -> ModelSet {
    toasty::models!(crate::*)
}

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
pub enum Error {
    #[error(transparent)]
    Db(#[from] toasty::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),
    #[error("Runtime Error: {0}")]
    Runtime(String),
}

pub async fn db() -> Result<toasty::Db, Error> {
    let project_dir = ProjectDirs::from("org", "quotidian", "propagation-notebook")
        .ok_or_else(|| Error::Runtime("Unable to determine project data directory".to_string()))?
        .data_dir()
        .to_path_buf();
    std::fs::create_dir_all(&project_dir)?;
    let db_uri = match std::env::var("PN_DB_URI") {
        Ok(s) => Ok(s),
        Err(std::env::VarError::NotPresent) => Ok(format!(
            "sqlite:{}",
            project_dir
                .join("propagation-notebook.sqlite")
                .to_str()
                .unwrap()
        )),
        e => e,
    }?;
    Ok(toasty::Db::builder()
        .models(models())
        .connect(&db_uri)
        .await?)
}

pub trait ImportProgressReporter {
    fn begin_step(&mut self, name: &str, total: usize);
    fn increment(&mut self);
    fn finish_step(&mut self);
}
