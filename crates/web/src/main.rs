use clap::Parser;
use libpropagation::region::Region;
use serde::Serialize;
use tokio::net::TcpListener;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::{BaseUrl, Cx},
    cookie::{CookieLayer, Key},
    metadata::Metadata,
    router::{
        Compression, Router, RouterBuilderDiscoverExt, StatusCode, content::Json, not_found, page,
        route,
    },
    runtime::RuntimeLayer,
    view::{View, view},
};
use tracing::debug;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

use crate::{config::AppConfig, context::db, error::Error, tasks::background_tasks};

mod actions;
mod admin;
mod assets;
mod auth;
mod citation;
#[allow(dead_code)]
mod components;
mod config;
mod context;
mod crumb;
mod error;
mod flash;
mod home;
mod propagation;
mod regions;
mod shell;
mod tasks;
mod taxa;
mod users;
mod util;

#[derive(Debug, clap::Parser)]
#[command(version)]
pub struct Options {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    #[command(about = "Run the web application")]
    Run,
    #[command(about = "Generate a new configuration file template")]
    GenerateConfig,
}

const SITE_NAME: &str = "Propagation Notebook";

impl Subtitle for Metadata {
    fn subtitle<S: AsRef<str>>(self, title: S) -> Self {
        self.title(format!("{} | {SITE_NAME}", title.as_ref()))
    }
}

trait Subtitle {
    fn subtitle<S: AsRef<str>>(self, title: S) -> Self;
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();
    let options = Options::parse();

    let res = match options.command {
        Some(Command::GenerateConfig) => {
            let template =
                confique::yaml::template::<AppConfig>(confique::yaml::FormatOptions::default());
            println!("{template}");
            Ok(())
        }
        None | Some(Command::Run) => run().await,
    };

    if let Err(err) = res {
        eprintln!("{}", err);
        if let Error::Configuration(_) = err {
            eprintln!(
                "To generate a default configuration file template, re-run this program with the 'generate-config' command argument",
            )
        }
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let db = libpropagation::db(true).await?;
    let config = AppConfig::load().map_err(|e| Error::Configuration(e.to_string()))?;
    debug!(?config);

    if config.enable_background_tasks {
        debug!("Enabling background tasks...");
        tokio::spawn(background_tasks(db.clone()));
    }

    let listener = TcpListener::bind((config.listen.host, config.listen.port)).await?;

    let mut service_builder = Router::builder()
        .discover()
        .assets(AssetBundle::load()?)
        .with(db)
        .with(Key::generate())
        .with(BaseUrl::new(config.base_url))
        .layer(RuntimeLayer::new())
        .layer(CookieLayer::new());

    if let Some(compression) = config.enable_compression {
        service_builder = service_builder.with(if compression {
            Compression::default()
        } else {
            Compression::disable()
        });
    }

    let service = service_builder.build();

    topcoat::serve(listener, service).await?;

    Ok(())
}

not_found!("/");

#[page("/about", with(Metadata::new().subtitle("About This Site")))]
async fn about(cx: &Cx) -> topcoat::Result<impl View> {
    Ok(view! {
        <h1>"About Propagation Notebook"</h1>
        <div id="content" class="flex flex-col gap-4 text-lg">
            <div>
                <h3>"Why This Site Exists"</h3>
                <div class="my-4">
                    r#"There is no single reference for propagating native
                    plants. Some species are well studied; many are not.
                    Information is scattered across seed producers, academic
                    studies, regional guides, and advice passed between
                    restoration professionals. Propagation Notebook aims to
                    bring it together as a practical reference for collecting
                    and propagating native plants for ecological restoration."#
                </div>
            </div>
            <div>
                <h3>"What You'll Find"</h3>
                <div class="my-4">
                    r#"Search out taxonomic database based by scientific or
                    common name, then explore propagation protocols assigned to
                    individual species. Protocols cover collecting, cleaning,
                    and germinating seed, with citations where available."#
                </div>
                <div class="my-4">
                    r#"Because seed timing depends on location, species are also
                    organized into regions. Each region tracks species status and
                    estimated harvest windows based on iNaturalist observations.
                    Coverage currently focuses on parts of the upper Midwest,
                    with more regions planned."#
                </div>
            </div>
            <div>
                <h3>"Our Goal"</h3>
                <div class="my-4">
                    r#"To become a comprehensive reference for native plant
                    propagation, useful to home gardeners and restoration
                    professionals alike. The site is still early in development
                    but will hopefully improve as more data is added and as we
                    add new features that will allow community contributions."#
                </div>
            </div>
        </div>
    })
}

#[derive(Debug, Serialize)]
enum HealthStatus {
    Healthy,
    Unhealthy,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: HealthStatus,
    version: &'static str,
    message: Option<String>,
}

#[route(GET "/api/health")]
async fn health(cx: &Cx) -> topcoat::Result<(StatusCode, Json<HealthResponse>)> {
    Ok(match Region::all().one().exec(&mut db(cx)).await {
        Ok(_) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: HealthStatus::Healthy,
                version: env!("CARGO_PKG_VERSION"),
                message: None,
            }),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: HealthStatus::Unhealthy,
                version: env!("CARGO_PKG_VERSION"),
                message: Some(e.to_string()),
            }),
        ),
    })
}
