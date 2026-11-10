use std::hash::{DefaultHasher, Hash, Hasher};

use libpropagation::{auth::PermissionCode, taxonomy::Taxon};
use topcoat::{
    asset::{Asset, asset_config},
    context::Cx,
    icon::icon,
    metadata::Metadata,
    router::{
        Slot, StatusCode,
        error::{ForbiddenError, NotFoundError, UnauthorizedError},
        href, layout,
        request::uri,
    },
    runtime::{Event, shard, signal},
    tailwind,
    view::{
        Attributes, Child, View, ViewExt, attributes, class, component, element_id, error_boundary,
        view,
    },
};

use crate::{
    SITE_NAME, admin,
    assets::{
        CLOSE_DROPDOWN_JS, FONT_BODY, FONT_HEAD, HEADER_IMAGES, HYDRASTIS_CANADENSIS, LEAFLET_CSS,
        LEAFLET_JS, mdi,
    },
    auth,
    components::{avatar::*, dropdown_menu::*, floating::FloatingAlign, input::input},
    context::{current_user, db},
    flash, home, regions, taxa, users,
};

// get a stable but "random" header image for each page so we have a variety of
// headers but the header of a particular page won't change on reload. Hash the
// url and convert it to an integer and use that to index into the header image
// collection. Special case the landing pages (home, login) to a particular
// image.
fn header_image_for_page(cx: &Cx) -> &Asset {
    let uri = uri(cx);
    if href!(home).is_current(cx) || href!(auth::login).is_current(cx) {
        return &HYDRASTIS_CANADENSIS;
    }
    let mut hasher = DefaultHasher::new();
    uri.hash(&mut hasher);
    let hash_value = hasher.finish();
    HEADER_IMAGES[(hash_value as usize) % HEADER_IMAGES.len()]
}

#[shard("/taxa/search/results")]
pub async fn taxon_search_results(
    cx: &Cx,
    query_string: String,
    visible: bool,
) -> topcoat::Result<impl View> {
    let nothing = view! {}.boxed();
    if !visible || query_string.len() < 2 {
        return Ok(nothing);
    }
    let mut db = db(cx);
    const LIMIT: u64 = 40;
    let query = Taxon::filter(Taxon::search_filter(&query_string))
        .include(Taxon::fields().photo())
        .order_by((
            Taxon::fields().sequence().asc(),
            Taxon::fields().complete_name().asc(),
        ));
    let total = query.clone().count().exec(&mut db).await?;
    let taxa = query.limit(LIMIT as usize).exec(&mut db).await?;
    if taxa.is_empty() {
        return Ok(nothing);
    }
    Ok(view! {
        <div
            class="flex flex-col p-3 gap-2
                fixed left-3 right-3
                md:absolute md:left-0 md:right-0 md:mx-0
                max-h-100 overflow-y-auto
                text-sm text-foreground bg-background/80 z-50
                rounded-xl border border-border shadow-sm"
        >
            <ul class="contents">
                for taxon in taxa.iter() {
                    <li class="py-1">
                        <span class="latin">
                            <a href=(href!(taxa::details, taxa::taxon_id: taxon.id))>
                                (&taxon.complete_name)
                            </a>
                        </span>
                    </li>
                }
                if total > LIMIT {
                    <li>(format!("...and {} more", total - LIMIT))</li>
                }
            </ul>
        </div>
    }
    .boxed())
}

#[component]
pub async fn taxon_search_bar(
    cx: &Cx,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    let query_string = signal(cx, String::new);
    let search_focus = signal(cx, || false);
    Ok(view! {
        <div
            @focusin=$(|_e: Event| search_focus.set(true))
            @focusout=$(|_e: Event| search_focus.set(false))
            class=(class!("relative", attrs.remove("class")))
            (attrs)
        >
            <form method="get" action=(href!(taxa::search)) class="contents">
                input(
                    attrs: attributes! {
                        type="text"
                        name="q"
                        placeholder="Search for a taxon"
                        autocomplete="off"
                        @input=$(|e: Event| query_string.set(e.target.value))
                    }
                )
                taxon_search_results(
                    query_string: $(query_string.get()),
                    visible: $(search_focus.get())
                )
            </form>
        </div>
    })
}

#[layout(
    "/",
    with(Metadata::new()
        .title(SITE_NAME)
        .description(
            "A comprehensive reference guide for growing and collecting native plants for ecological restoration",
        ))
)]
async fn shell(cx: &Cx, slot: Slot<'_>) -> topcoat::Result<impl View> {
    let header_bg = header_image_for_page(cx);

    Ok(view! {
        <!DOCTYPE html>
        <html
            style=(format!(
                "--background-image:url('{}')",
                asset_config(cx).resolve(cx, *header_bg),
            ))
        >
            <head>
                topcoat::metadata::tags()
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
                    if href!(auth::login).is_current(cx) {
                        (slot)
                    } else {
                        full_page_layout((slot))
                    }
                </div>
            </body>
        </html>
    })
}

#[component]
pub async fn user_menu(cx: &Cx) -> topcoat::Result<impl View> {
    let menu_id = element_id(cx);
    Ok(view! {
        <li class="ml-auto">
            if let Some(user) = current_user(cx).await {
                <form
                    method="POST"
                    action=(href!(auth::do_logout))
                    id="logoutForm"
                    class="hidden"
                ></form>
                dropdown_menu(
                    dropdown_menu_trigger(
                        target: &menu_id,
                        attrs: attributes! { class="flex" },
                        avatar(
                            attrs: attributes! { class="me-2 bg-background/60" },
                            size: AvatarSize::Sm,
                            avatar_fallback(icon(data: mdi::ACCOUNT))
                        )
                        <span class="hidden md:inline-block">(&user.username)</span>
                    )
                    dropdown_menu_content(
                        id: &menu_id,
                        align: FloatingAlign::End,
                        dropdown_menu_label(
                            "Logged in as "
                            (&user.username)
                        )
                        dropdown_menu_separator()
                        dropdown_menu_navigation_item(
                            close: Some(&menu_id),
                            attrs: attributes! { href=(href!(users::account)) type="button" },
                            "My Account"
                        )
                        if user.has_permission(PermissionCode::AdminConsole) {
                            dropdown_menu_navigation_item(
                                close: Some(&menu_id),
                                attrs: attributes! { href=(href!(admin::console)) },
                                "Admin Console"
                            )
                        }
                        dropdown_menu_item(
                            close: Some(&menu_id),
                            attrs: attributes! { type="submit" class="text-destructive" form="logoutForm" },
                            "Log Out"
                        )
                    )
                )
            } else {
                <a
                    class="flex"
                    href=(href!(auth::login).query(
                        auth::LoginFormParams { redirect: Some(uri(cx).to_string()) },
                    ))
                >
                    "Log in"
                </a>
            }
        </li>
    })
}

#[component]
async fn full_page_layout(cx: &Cx, child: Child<'_>) -> topcoat::Result<impl View> {
    let uri = uri(cx);
    let flash_msg = flash::pop_message(cx)?;

    Ok(view! {
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
            if let Some(msg) = flash_msg {
                flash::flash_alert(attrs: attributes! { class="my-3" }, flash: &msg)
            }
            error_boundary(
                fallback: |error| {
                    match error {
                        e if e.downcast_ref::<UnauthorizedError>().is_some() => {
                            Ok(
                                view! {
                                    (StatusCode::UNAUTHORIZED)
                                    <h1>"401 — Unauthorized"</h1>
                                    <div>
                                        "Sorry, you must be logged in to view this page. Please "
                                        <a href=(href!(auth::login))>"log in"</a>
                                        " to continue."
                                    </div>
                                }.boxed(

                                ),
                            )
                        }
                        e if e.downcast_ref::<ForbiddenError>().is_some() => {
                            Ok(
                                view! {
                                    (StatusCode::FORBIDDEN)
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
                                    (StatusCode::NOT_FOUND)
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
                (child)
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
    })
}
