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
        Compression, Router, RouterBuilderDiscoverExt, StatusCode, content::Json, href, not_found,
        page, route,
    },
    runtime::RuntimeLayer,
    view::{View, attributes, view},
};
use tracing::debug;

use crate::{
    components::{button::*, input::input},
    config::AppConfig,
    context::db,
    error::Error,
    tasks::background_tasks,
};

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
    tracing_subscriber::fmt::init();
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

#[page("/")]
async fn home(cx: &Cx) -> topcoat::Result<impl View> {
    Ok(view! {
        <div class="flex flex-col gap-4">
            <hgroup>
                <h1>"Propagation Notebook"</h1>
                <p>
                    "A comprehensive reference guide for growing and collecting native plants for ecological restoration."
                </p>
            </hgroup>
            <section>
                <h2>"Growing Native Plants"</h2>
                <p>"Find propagation information for a particular species:"</p>
                <form
                    method="get"
                    action=(href!(taxa::search))
                    class="flex my-6 w-full md:w-xl"
                >
                    input(
                        attrs: attributes! {
                            type="text"
                            name="q"
                            placeholder="Search for a taxon"
                            class="me-2 grow"
                        }
                    )
                    button(attrs: attributes! { type="submit" }, "Search")
                </form>
            </section>
            <section>
                <h2>"Regional Information"</h2>
                <p>
                    "Find out which plants are in your "
                    <a href=(href!(regions::list))>"region"</a>
                    " and when they are bearing fruit."
                </p>
            </section>
        </div>
    })
}

#[page("/about")]
async fn about(cx: &Cx) -> topcoat::Result<impl View> {
    Ok(view! {
        <h1>"About This Site"</h1>
        <div id="content" class="flex flex-col gap-4 text-lg">
            <div>
                <h3>"The Problem"</h3>
                <div class="my-4">
                    r#"There is not currently any single site that serves as a
                    reference for how to propagate a particular species.  Native
                    species tend to be more difficult to propagate and some
                    species are still poorly understood. There is currently
                    no single reference to consult when you want to know how to
                    propagate a specific species. For commercially-available
                    species, many people use the information provided by
                    the seed producer. For species that are not available
                    commercially, you generally have to search for studies
                    or species-specific references. For more obscure or
                    tricky-to-propagate species, ecological restoration
                    professionals sometimes pass tips and tricks by word of
                    mouth. "#
                </div>
            </div>
            <div>
                <h3>"The Goal"</h3>
                <div class="my-4">
                    r#"That's where this website comes in. The goal of this
                    site is to become a comprehensive reference to collecting
                    seeds and propagating native plants  for use in ecological
                    restoration. In its current state, this site is fairly
                    limited, but it is beginning to be useful. "#
                </div>
            </div>
            <div>
                <h3>"Features"</h3>
                <div class="my-4">
                    r#"There are a couple main focuses of this site. The first
                    is propagation protocols. Because many species have very
                    similar propagation protocols (e.g. "30 days of cold moist
                    stratification"), these are modeled as general protocols that
                    can be assigned to particular species. We hope to eventually
                    collect propagation instructions for as many species as
                    possible. This part of the site is intended for use by both
                    home gardeners and ecological professionals.  "#
                </div>
                <div class="my-4">
                    r#"The second main focus is seed collection. In order
                    to restore habitat, you first need the propagation material.
                    In many cases the best and easiest choice is seed. But
                    realiable information about collecting, processing and
                    storing the seeds is dispersed and often hard to find. This
                    is especially true for species that are not available in the
                    commercial trade. So our goal is to be a comprehensive
                    reference for both how to collect the seed, as well as
                    *when* to collect the seed."#
                </div>
                <div class="my-4">
                    r#"Of course, the seed collection window is highly dependent
                    on your geographical location, so there is a concept of
                    "regions". Each region has a checklist of species and
                    information about the status of that species within the
                    region. One of the most relevant pieces of information is the
                    dates that the species produces fruit or seeds within that
                    region. At the moment, this site has a fairly limited set
                    of regions in the upper midwest. In the future, we hope
                    to expand this to many other regions and allow community
                    management of this information. For now, it is centrally
                    managed until I can get the infrastructure in place to
                    support external contributions."#
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
