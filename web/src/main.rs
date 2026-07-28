use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{Router, RouterBuilderDiscoverExt, layout, page},
    tailwind,
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
            .assets(AssetBundle::load_dir("target/assets")?)
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
                <title>"Propagation Notebook"</title>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
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
            <body class="flex flex-col min-h-screen">
                <header
                    class="flex items-center justify-between font-bold flex-wrap p-6"
                >
                    <div class="flex items-center flex-shrink-0 text-white mr-8">
                        <a href="/">"Propagation Notebook"</a>
                    </div>
                    <nav class="w-full block flex-grow lg:w-auto">
                        <ul class="lg:flex gap-x-6">
                            <li>
                                <a
                                    class="block mt-6 lg:mt-0 text-orange-100 hover:text-white"
                                    href="/taxa"
                                >
                                    "Taxonomy"
                                </a>
                            </li>
                            <li>
                                <a
                                    class="block mt-6 lg:mt-0 text-orange-100 hover:text-white"
                                    href="/regions"
                                >
                                    "Regions"
                                </a>
                            </li>
                            <li>
                                <a
                                    class="block mt-6 lg:mt-0 text-orange-100 hover:text-white"
                                    href="/propagation"
                                >
                                    "Propagation Protocols"
                                </a>
                            </li>
                        </ul>
                    </nav>
                </header>
                <main class="m-6 flex-grow">(slot?)</main>
                <footer
                    class="flex items-center justify-between font-bold flex-wrap p-6"
                >
                    "Developed by Jonathon Jongsma"
                </footer>
            </body>
        </html>
    }
}

#[page("/")]
async fn home() -> topcoat::Result {
    view! { <h1>"Home"</h1> }
}
