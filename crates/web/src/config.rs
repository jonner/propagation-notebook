use confique::Config;

#[derive(Debug, Config)]
pub(crate) struct ListenConfig {
    #[config(default = "localhost", env = "PN_HOST")]
    pub(crate) host: String,
    #[config(default = 3000, env = "PN_PORT")]
    pub(crate) port: u16,
}

#[derive(Debug, Config)]
pub(crate) struct AppConfig {
    #[config(nested)]
    pub(crate) listen: ListenConfig,
    #[config(env = "PN_BASE_URL")]
    pub(crate) base_url: url::Url,
    #[config(default = false, env = "PN_ENABLE_BACKGROUND_TASKS")]
    pub(crate) enable_background_tasks: bool,
    #[config(env = "PN_ENABLE_COMPRESSION")]
    pub(crate) enable_compression: Option<bool>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not determine project directories. Is HOME set?")]
    ProjectDirectories,
    #[error(transparent)]
    Configuration(#[from] confique::Error),
}

impl AppConfig {
    pub(crate) fn load() -> Result<Self, Error> {
        const CONFIG_FILE_NAME: &str = "propagation-notebook.yml";
        let project_dirs =
            directories::ProjectDirs::from("org", "quotidian", "propagation-notebook")
                .ok_or_else(|| Error::ProjectDirectories)?;
        let global_config_file_path = "/etc/propagation-notebook/".to_string() + CONFIG_FILE_NAME;
        let user_config_file_path = project_dirs.config_dir().join(CONFIG_FILE_NAME);
        Self::builder()
            // Use configuration values from env variables if they exist
            .env()
            // then try a config file located in the current working directory
            .file(CONFIG_FILE_NAME)
            // then fall back to a user-level config file
            .file(user_config_file_path)
            // finally check for any system-level config file
            .file(global_config_file_path)
            .load()
            .map_err(Into::into)
    }
}
