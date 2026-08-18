use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    font::{Font, fontsource::fontsource_font},
    icon::{icon, iconify},
    router::{Router, RouterBuilderDiscoverExt, href, layout, not_found, page},
    tailwind,
    view::{attributes, view},
};

use crate::{error::Error, tasks::background_tasks};

mod citation;
mod components;
mod error;
mod leaflet;
mod propagation;
mod regions;
mod tasks;
mod taxa;
mod util;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let db = libpropagation::db().await?;
    tokio::spawn(background_tasks(db.clone()));
    topcoat::start(
        Router::builder()
            .discover()
            .assets(AssetBundle::load()?)
            .app_context(db)
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
                    <nav
                        class="w-full block flex-grow lg:w-auto flex items-center gap-6 text-white"
                    >
                        <ul class="contents">
                            <li>
                                <a class="block" href=(href!(home))>
                                    icon(
                                        data: mdi::FLOWER_POPPY,
                                        label: "Home",
                                        attrs: attributes! { class="icon" }
                                    )
                                    <span class="caption">"Propagation Notebook"</span>
                                </a>
                            </li>
                            <li>
                                <a class="block" href=(href!(taxa::list))>
                                    icon(
                                        data: mdi::FORMAT_LIST_BULLETED,
                                        label: "list icon",
                                        attrs: attributes! { class="icon" }
                                    )
                                    <span class="caption">"Taxonomy"</span>
                                </a>
                            </li>
                            <li>
                                <a class="block" href=(href!(regions::list))>
                                    icon(
                                        data: mdi::GLOBE,
                                        label: "list icon",
                                        attrs: attributes! { class="icon" }
                                    )
                                    <span class="caption">"Regions"</span>
                                </a>
                            </li>
                            // <li>
                            //     <a class="block" href=(href!(propagation::list))>
                            //         icon(
                            //             data: mdi::SPROUT,
                            //             label: "list icon",
                            //             attrs: attributes! { class="icon" }
                            //         )
                            //         <span class="caption">"Propagation Protocols"</span>
                            //     </a>
                            // </li>
                            // <li>
                            //     <a class="block" href=(href!(about))>
                            //         icon(
                            //             data: mdi::ABOUT,
                            //             label: "about icon",
                            //             attrs: attributes! { class="icon" }
                            //         )
                            //         <span class="caption">"About"</span>
                            //     </a>
                            // </li>
                        </ul>
                    </nav>
                </header>
                <main class="m-3 md:m-6 flex-grow">
                    match slot {
                        Ok(content) => (content),
                        Err(e) => {
                            <h1>"Error"</h1>
                            <div class="error p-6">(e.to_string())</div>
                        }
                    }
                </main>
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
                <form method="get" action="/taxa" class="flex my-6 w-full md:w-xl">
                    <input
                        type="text"
                        name="q"
                        placeholder="Search for a taxon"
                        class="me-2 flex-grow"
                    >
                    <button type="submit">"Search"</button>
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
