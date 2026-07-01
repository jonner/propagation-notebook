use directories::ProjectDirs;
use toasty::ModelSet;

use crate::error::Error;

pub mod collecting;
pub mod error;
pub mod propagation;
pub mod region;
pub mod taxonomy;

pub fn models() -> ModelSet {
    toasty::models!(crate::*)
}

pub async fn db() -> Result<toasty::Db, Error> {
    let project_dir = ProjectDirs::from("org", "quotidian", "propagation-notebook")
        .ok_or_else(|| Error::Runtime(error::Runtime::ProjectDirNotFound))?
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
        Err(e) => Err(Error::Runtime(error::Runtime::InvalidEnvVar(e.to_string()))),
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
pub mod dto {
    use serde::Serialize;

    #[derive(Serialize, Debug, Clone)]
    pub struct ObjectReference {
        pub id: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
    }

    impl std::fmt::Display for ObjectReference {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(name) = &self.name {
                write!(f, "{}: {}", self.id, name)
            } else {
                write!(f, "{}", self.id)
            }
        }
    }
}
