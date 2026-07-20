use std::{net::SocketAddr, sync::Arc};

use axum::{Router, response::IntoResponse, routing::get};
use clap::Parser;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub mod error;
pub mod propagation;
pub mod region;
pub mod taxonomy;
mod templates;

#[derive(Debug, Clone, clap::Parser)]
pub struct Cli {
    #[arg(short, long, help = "Port to listen on")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::default());
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let args = Cli::parse();
    App::new().await?.serve(args.port).await
}

#[derive(Debug, Clone)]
pub struct App {
    router: axum::Router,
}

pub struct AppState {
    db: toasty::Db,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            db: libpropagation::db().await?,
        })
    }
}

#[axum::debug_handler]
pub async fn handle_root() -> impl IntoResponse {
    templates::pages::root()
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            router: Router::new()
                .route("/", get(handle_root))
                .nest("/regions/", region::router())
                .nest("/propagation/", propagation::router())
                .nest("/taxa/", taxonomy::router())
                .with_state(Arc::new(AppState::new().await?)),
        })
    }
}

impl App {
    pub async fn serve(self, listen_port: u16) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(("0.0.0.0", listen_port)).await?;
        axum::serve(
            listener,
            self.router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(Into::into)
    }
}
