use topcoat::{
    router::{Router, RouterBuilderDiscoverExt, Slot, layout, page},
    view::view,
};

mod citation;
mod components;
mod error;
mod leaflet;
mod propagation;
mod regions;
mod taxa;
mod util;

use crate::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    topcoat::start(
        Router::builder()
            .discover()
            .app_context(libpropagation::db().await?)
            .build(),
    )
    .await?;
    Ok(())
}

#[layout("/")]
async fn layout(slot: topcoat::Result) -> topcoat::Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                topcoat::dev::script()
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <meta charset="UTF-8">
                <link
                    rel="stylesheet"
                    href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                    integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                    crossorigin=""
                >
                <script
                    src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                    integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                    crossorigin=""
                ></script>
            </head>
            <body>
                <nav>
                    <ul>
                        <li><a href="/">"Home"</a></li>
                        <li><a href="/taxa">"Taxonomy"</a></li>
                        <li><a href="/regions">"Regions"</a></li>
                        <li><a href="/propagation">"Propagation Protocols"</a></li>
                    </ul>
                </nav>
                (slot?)
            </body>
        </html>
    }
}

#[page("/")]
async fn home() -> topcoat::Result {
    view! { <h1>"Home"</h1> }
}
