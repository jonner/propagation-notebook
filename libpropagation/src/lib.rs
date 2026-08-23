use directories::ProjectDirs;
use toasty::{ModelSet, embed_migrations, migration::MigrationSet};
use tracing::{debug, trace};

use crate::error::Error;

pub mod citation;
pub mod collecting;
pub mod dto;
pub mod error;
pub mod propagation;
pub mod region;
pub mod taxonomy;

pub fn models() -> ModelSet {
    toasty::models!(crate::*)
}

static MIGRATIONS: MigrationSet = embed_migrations!("../db");
pub async fn db(migrate: bool) -> Result<toasty::Db, Error> {
    let db_uri = match std::env::var("PN_DB_URI") {
        Ok(s) => Ok(s),
        Err(std::env::VarError::NotPresent) => {
            let project_dir = ProjectDirs::from("org", "quotidian", "propagation-notebook")
                .ok_or_else(|| Error::Runtime(error::Runtime::ProjectDirNotFound))?
                .data_dir()
                .to_path_buf();
            std::fs::create_dir_all(&project_dir)?;
            Ok(format!(
                "sqlite:{}",
                project_dir
                    .join("propagation-notebook.sqlite")
                    .to_str()
                    .unwrap()
            ))
        }
        Err(e) => Err(Error::Runtime(error::Runtime::InvalidEnvVar(e.to_string()))),
    }?;
    trace!(?db_uri);
    let db = toasty::Db::builder()
        .models(models())
        .connect(&db_uri)
        .await?;
    if migrate {
        let report = MIGRATIONS.apply(&db).await?;
        debug!("Applied {} migrations", report.applied());
    }
    Ok(db)
}

pub trait ImportProgressReporter {
    fn begin_step(&mut self, name: &str, total: usize);
    fn increment(&mut self);
    fn finish_step(&mut self);
}
