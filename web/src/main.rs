use std::hash::{DefaultHasher, Hash, Hasher};

use tokio::net::TcpListener;
use topcoat::{
    asset::{Asset, AssetBundle, RouterBuilderAssetExt, asset_config},
    context::Cx,
    cookie::RouterBuilderCookieExt,
    icon::icon,
    router::{
        Router, RouterBuilderDiscoverExt, Slot,
        error::{ForbiddenError, NotFoundError, UnauthorizedError},
        href, layout, not_found, page,
        request::uri,
    },
    runtime::RouterBuilderRuntimeExt,
    session::{RouterBuilderSessionExt, SessionConfig},
    tailwind,
    view::{View, ViewExt, attributes, error_boundary, view},
};
use tracing::debug;

use crate::{
    assets::{
        CLOSE_DROPDOWN_JS, FONT_BODY, FONT_HEAD, HEADER_IMAGES, HYDRASTIS_CANADENSIS, LEAFLET_CSS,
        LEAFLET_JS, mdi,
    },
    auth::login,
    components::{
        button::*,
        input::input,
        pn::{taxon_search_bar, user_menu},
    },
    config::AppConfig,
    error::Error,
    tasks::background_tasks,
};

mod assets;
mod auth;
mod citation;
mod components;
mod config;
mod context;
mod error;
mod propagation;
mod regions;
mod tasks;
mod taxa;
mod users;
mod util;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let db = libpropagation::db(true).await?;
    let config = AppConfig::load().map_err(|e| Error::Configuration(e.to_string()))?;

    if config.enable_background_tasks {
        debug!("Enabling background tasks...");
        tokio::spawn(background_tasks(db.clone()));
    }

    let listener = TcpListener::bind((config.listen.host, config.listen.port)).await?;

    let service = Router::builder()
        .runtime()
        .discover()
        .assets(AssetBundle::load()?)
        .app_context(db)
        .cookies()
        .sessions(SessionConfig::default())
        .base_url(config.base_url.to_string())
        .build();

    topcoat::serve(listener, service).await?;

    Ok(())
}

// get a stable but "random" header image for each page so we have a variety of
// headers but the header of a particular page won't change on reload. Hash the
// url and convert it to an integer and use that to index into the header image
// collection. Special case the landing pages (home, login) to a particular
// image.
fn header_image_for_page(cx: &Cx) -> &Asset {
    let uri = uri(cx);
    if href!(home).is_current(cx) || href!(login).is_current(cx) {
        return &HYDRASTIS_CANADENSIS;
    }
    let mut hasher = DefaultHasher::new();
    uri.hash(&mut hasher);
    let hash_value = hasher.finish();
    HEADER_IMAGES[(hash_value as usize) % HEADER_IMAGES.len()]
}

not_found!("/");

#[layout("/")]
async fn layout(cx: &Cx, slot: Slot<'_>) -> topcoat::Result<impl View> {
    let uri = uri(cx);
    let header_bg = header_image_for_page(cx);

    Ok(view! {
        <!DOCTYPE html>
        <html
            style=(format!(
                "--background-image:url('{}')",
                asset_config(cx).resolve(*header_bg),
            ))
        >
            <head>
                topcoat::runtime::script()
                topcoat::dev::script()
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <meta charset="UTF-8">
                <title>"Propagation Notebook"</title>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                <link rel="stylesheet" href=(LEAFLET_CSS)>
                <script src=(LEAFLET_JS)></script>
                <script src=(CLOSE_DROPDOWN_JS)></script>
                topcoat::font::link(font: FONT_HEAD)
                topcoat::font::link(font: FONT_BODY)
            </head>
            <body>
                <div class="main-content">
                    <header
                        class="dark flex flex-col items-start justify-between font-bold flex-wrap bg-(image:--background-image) bg-center bg-cover h-[8rem] md:h-[12rem]"
                    >
                        <nav
                            class="text-foreground bg-background/70 w-full block shrink flex items-center gap-6 md:gap-4 px-3 md:px-6 py-2"
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
                                    <a class="block" href=(href!(taxa::explore))>
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
                                <li class="md:px-6 mx-auto grow lg:max-w-1/2">
                                    taxon_search_bar()
                                </li>
                                user_menu()
                            </ul>
                        </nav>
                    </header>
                    <main class="m-3 md:m-6 grow">
                        error_boundary(
                            fallback: |error| {
                                match error {
                                    e if e.downcast_ref::<UnauthorizedError>().is_some() => {
                                        Ok(
                                            view! {
                                                <h1>"401 — Unauthorized"</h1>
                                                <div>
                                                    "Sorry, you must be logged in to view this page. Please "
                                                    <a href=(href!(login))>"log in"</a>
                                                    " to continue."
                                                </div>
                                            }.boxed(

                                            ),
                                        )
                                    }
                                    e if e.downcast_ref::<ForbiddenError>().is_some() => {
                                        Ok(
                                            view! {
                                                <h1>"403 — Access Denied"</h1>
                                                <div>
                                                    (format!(
                                                        "Sorry, you do not have permission to access '{uri}' on this server.",
                                                    ))
                                                </div>
                                            }.boxed(

                                            ),
                                        )
                                    }
                                    e if e.downcast_ref::<NotFoundError>().is_some() => {
                                        Ok(
                                            view! {
                                                <h1>"404 Not Found"</h1>
                                                <p>
                                                    (format!(
                                                        "We're sorry, but the page '{uri}' cannot be found. The link might be outdated, or the URL could have been a typo.",
                                                    ))
                                                </p>
                                            }.boxed(

                                            ),
                                        )
                                    }
                                    e => Err(e),
                                }
                            },
                            (slot)
                        )
                    </main>
                    <footer
                        class="dark bg-(image:--background-image) bg-center bg-cover mt-6 shadow-[0_-1rem_2rem_rgba(0,0,0,0.2)]"
                    >
                        <div
                            class="text-foreground bg-linear-to-t from-background to-50% to-transparent block px-3 md:px-6 pt-50 pb-6"
                        >
                            "Developed with "
                            icon(
                                data: mdi::HEART,
                                label: "Love",
                                attrs: attributes! { class="text-red-300 inline-block" }
                            )
                            " by volunteers"
                            <div class="opacity-60">
                                <div>
                                    "Taxonomy based on "
                                    <a href="https://www.itis.gov">"ITIS"</a>
                                </div>
                                <div>
                                    "Phenology data provided by "
                                    <a href="https://inaturalist.org">"iNaturalist.org"</a>
                                </div>
                            </div>
                        </div>
                    </footer>
                </div>
            </body>
        </html>
    })
}

#[page("/")]
async fn home() -> topcoat::Result<impl View> {
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
async fn about() -> topcoat::Result<impl View> {
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
