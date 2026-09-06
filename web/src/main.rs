use topcoat::{
    asset::{Asset, AssetBundle, RouterBuilderAssetExt, asset, asset_config},
    context::Cx,
    font::{Font, fontsource::fontsource_font},
    icon::{icon, iconify},
    router::{
        Router, RouterBuilderDiscoverExt,
        error::{ForbiddenError, NotFoundError},
        href, layout, not_found, page,
        request::uri,
    },
    tailwind,
    view::{attributes, class, view},
};
use tracing::debug;

use crate::{
    components::{button::button, input::input},
    error::Error,
    tasks::background_tasks,
};

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
    let db = libpropagation::db(true).await?;
    if std::env::var("ENABLE_BACKGROUND_TASKS").is_ok() {
        debug!("Enabling background tasks...");
        tokio::spawn(background_tasks(db.clone()));
    }
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

not_found!("/");

// All images from iNaturalist licensed in the public domain
const HEADERS: &[Asset] = &[
    // https://www.inaturalist.org/photos/210648286
    asset!("assets/cypripedium-reginae.webp"),
    // https://www.inaturalist.org/photos/12597071
    asset!("assets/asclepias-incarnata.webp"),
    // https://www.inaturalist.org/photos/147597352
    asset!("assets/empetrum-nigrum.webp"),
    // https://www.inaturalist.org/photos/135132632
    asset!("assets/escobaria-vivipara.webp"),
    // https://www.inaturalist.org/photos/102029239
    asset!("assets/hamamelis-virginiana.webp"),
    // https://www.inaturalist.org/photos/531026295
    asset!("assets/desmanthus-illinoensis.webp"),
    // https://www.inaturalist.org/photos/690459895
    asset!("assets/hydrastis-canadensis.webp"),
];

#[layout("/")]
async fn layout(cx: &Cx, slot: topcoat::Result) -> topcoat::Result {
    let header_bg = HEADERS[rand::random_range(0..HEADERS.len())];
    let uri = uri(cx);
    let content = match slot {
        Err(e) if e.downcast_ref::<NotFoundError>().is_some() => view! {
            <h1>"404 Not Found"</h1>
            <p>
                (format!(
                    "We're sorry, but the page '{uri}' cannot be found. The link might be outdated, or the URL could have been a typo.",
                ))
            </p>
        },
        Err(e) if e.downcast_ref::<ForbiddenError>().is_some() => view! {
            <h1>"403 — Access Denied"</h1>
            <div>
                (format!(
                    "Sorry, you do not have permission to access '{uri}' on this server.",
                ))
            </div>
        },
        content => content,
    }?;
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
            <body>
                <div class="main-content">
                    <header
                        class=(class!(
                            "flex",
                            "flex-col",
                            "items-start",
                            "justify-between",
                            "font-bold",
                            "flex-wrap",
                            "bg-(image:--background-image)",
                            "bg-center",
                            "bg-cover",
                            "h-[8rem]",
                            "md:h-[12rem]",
                        ))
                        style=(format!(
                            "--background-image:url('{}')",
                            asset_config(cx).resolve(header_bg),
                        ))
                    >
                        <nav
                            class="w-full block shrink flex items-center gap-6 px-6 py-3 text-white bg-neutral-800/50"
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
                                    <a class="block" href=(href!(taxa::taxonomy))>
                                        icon(
                                            data: mdi::FORMAT_LIST_BULLETED,
                                            label: "Taxonomy",
                                            attrs: attributes! { class="icon" }
                                        )
                                        <span class="caption">"Taxonomy"</span>
                                    </a>
                                </li>
                                <li>
                                    <a class="block" href=(href!(regions::list))>
                                        icon(
                                            data: mdi::GLOBE,
                                            label: "Regions",
                                            attrs: attributes! { class="icon" }
                                        )
                                        <span class="caption">"Regions"</span>
                                    </a>
                                </li>
                                <li class="grow">
                                    <form
                                        method="get"
                                        action=(href!(taxa::search))
                                        class="flex gap-3"
                                    >
                                        input(
                                            attrs: attributes! { type="text" name="q" placeholder="Search for a taxon" }
                                        )
                                        button(
                                            attrs: attributes! { class="!hidden md:!inline-flex" type="submit" },
                                            "Search"
                                        )
                                    </form>
                                </li>
                            </ul>
                        </nav>
                    </header>
                    <main class="m-3 md:m-6 grow">(content)</main>
                    <footer>
                        "Developed with "
                        icon(
                            data: mdi::HEART,
                            label: "Love",
                            attrs: attributes! { class="text-red-300 inline-block" }
                        )
                        " by volunteers"
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
                </div>
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
