use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    font::{Font, fontsource::fontsource_font},
    icon::{icon, iconify},
    router::{Router, RouterBuilderDiscoverExt, layout, page},
    tailwind,
    view::{attributes, view},
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
            .assets(AssetBundle::load()?)
            .app_context(libpropagation::db().await?)
            .build(),
    )
    .await?;
    Ok(())
}

iconify::include!("mdi");
const FONT_HEAD: Font = fontsource_font!(AVERIA_SERIF_LIBRE, host: Asset);
const FONT_BODY: Font = fontsource_font!(AVERIA_SANS_LIBRE, host: Asset);

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
                topcoat::font::link(font: FONT_HEAD)
                topcoat::font::link(font: FONT_BODY)
            </head>
            <body class="flex flex-col min-h-screen">
                <header
                    class="flex items-center justify-between font-bold flex-wrap p-6"
                >
                    <div class="flex items-center flex-shrink-0 text-white mr-8">
                        <a href="/">
                            icon(
                                data: mdi::FLOWER_POPPY,
                                label: "Home",
                                attrs: attributes! { class="icon" }
                            )
                            "Propagation Notebook"
                        </a>
                    </div>
                    <nav class="w-full block flex-grow lg:w-auto">
                        <ul class="lg:flex gap-x-6">
                            <li>
                                <a
                                    class="block mt-6 lg:mt-0 text-orange-100 hover:text-white"
                                    href="/taxa"
                                >
                                    icon(
                                        data: mdi::FORMAT_LIST_BULLETED,
                                        label: "list icon",
                                        attrs: attributes! { class="icon" }
                                    )
                                    "Taxonomy"
                                </a>
                            </li>
                            <li>
                                <a
                                    class="block mt-6 lg:mt-0 text-orange-100 hover:text-white"
                                    href="/regions"
                                >
                                    icon(
                                        data: mdi::GLOBE,
                                        label: "list icon",
                                        attrs: attributes! { class="icon" }
                                    )
                                    "Regions"
                                </a>
                            </li>
                            <li>
                                <a
                                    class="block mt-6 lg:mt-0 text-orange-100 hover:text-white"
                                    href="/propagation"
                                >
                                    icon(
                                        data: mdi::SPROUT,
                                        label: "list icon",
                                        attrs: attributes! { class="icon" }
                                    )
                                    "Propagation Protocols"
                                </a>
                            </li>
                            <li>
                                <a
                                    class="block mt-6 lg:mt-0 text-orange-100 hover:text-white"
                                    href="/about"
                                >
                                    icon(
                                        data: mdi::ABOUT,
                                        label: "about icon",
                                        attrs: attributes! { class="icon" }
                                    )
                                    "About"
                                </a>
                            </li>
                        </ul>
                    </nav>
                </header>
                <main class="m-6 flex-grow">(slot?)</main>
                <footer class="p-6">
                    "Developed by Jonathon Jongsma"
                    <div class="text-sm text-white/50">
                        <div>
                            "Taxonomy based on "
                            <a href="https://www.itis.gov">"ITIS"</a>
                        </div>
                        <div>
                            "Phenology data provided by "
                            <a href="https://inaturalist.org">"iNaturalist.org"</a>
                        </div>
                    </div>
                </footer>
            </body>
        </html>
    }
}

#[page("/")]
async fn home() -> topcoat::Result {
    view! {
        <h1>"Native Plant Propagation Notebook"</h1>
        <div>
            "A reference for collecting and propagating native plants for ecological restoration"
        </div>
        <div class="card">
            <h2>"Propagation"</h2>
            <div>"Search for information about propagating a particular species"</div>
            <form method="get" action="/taxa" class="flex-grow mb-6">
                <input
                    type="text"
                    name="q"
                    placeholder="Search for a taxon"
                    class="me-2"
                >
                <button type="submit">"Search"</button>
            </form>
        </div>
        <div class="card">
            <h2>"Collect Seeds"</h2>
            <div>
                "Find out when plants are bearing seed in your "
                <a href="/regions">"region"</a>
                "."
            </div>
        </div>
    }
}

#[page("/about")]
async fn about() -> topcoat::Result {
    view! {
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
    }
}
