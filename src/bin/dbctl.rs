use anyhow::Context;
use toasty_cli::{Config, ToastyCli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;

    let uri = std::env::var("DB_URI").with_context(
        || "Please set DB_URI to the database connection URI (e.g. 'sqlite:./file.sqlite')",
    )?;
    let db = toasty::Db::builder()
        .models(propagation_notebook::models())
        .connect(&uri)
        .await?;

    let cli = ToastyCli::with_config(db, config);
    cli.parse_and_run().await?;

    Ok(())
}
