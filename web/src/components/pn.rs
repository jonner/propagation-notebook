use jiff::ToSpan;
use libpropagation::{
    region::{ConservationStatus, Origin},
    taxonomy::Taxon,
};
use topcoat::{
    asset::{Asset, asset},
    context::Cx,
    icon::icon,
    router::{href, request::uri},
    runtime::{Event, shard, signal},
    view::{
        AttributeValueViewParts, Attributes, Child, View, ViewExt, attributes, class, component,
        view,
    },
};
use uuid::Uuid;

use crate::{
    auth::{LoginFormParams, do_logout, login},
    components::{
        avatar::*, badge::*, breadcrumb::*, dropdown_menu::*, input::input, pagination::*,
        tooltip::*,
    },
    context::{current_user, db},
    mdi,
    taxa::{self, TaxonId},
    util::{ModifyOffset, PageState},
};

#[component]
pub async fn origin_badge(
    origin: Origin,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    let vals = match origin {
        Origin::Introduced => Some(("introduced", "IN", "Introduced")),
        Origin::Unknown => Some(("unknown", "UN", "Unknown origin")),
        Origin::Native => None,
    };
    Ok(if let Some((klass, text, tooltip_text)) = vals {
        view! {
            tooltip(
                <div class=(class!(klass, "badge", attrs.remove("class"))) (attrs)>
                    (text)
                </div>
                tooltip_content((tooltip_text))
            )
        }
        .boxed()
    } else {
        view! {}.boxed()
    })
}

#[component]
pub async fn conservation_status_badge(
    status: ConservationStatus,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    let (klass, text, tooltip_text) = match status {
        ConservationStatus::Endangered => ("endangered", "EN", "Endangered"),
        ConservationStatus::Threatened => ("threatened", "TH", "Threatened"),
        ConservationStatus::SpecialConcern => ("specialconcern", "SC", "Special Concern"),
    };
    Ok(view! {
        tooltip(
            <div class=(class!(klass, "badge", attrs.remove("class"))) (attrs)>
                (text)
            </div>
            tooltip_content((tooltip_text))
        )
    })
}

#[component]
pub async fn ancestor_breadcrumbs<L, P>(
    items: &[&Taxon],
    link_fn: L,
    #[default] link_final: bool,
    #[default(Some(2))] ellipsize: Option<usize>,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View>
where
    L: Fn(&Taxon) -> P + Send + Sync,
    P: AttributeValueViewParts,
{
    let mut item_iter = items.iter();
    let last = item_iter.next_back();
    let root = item_iter.next();
    let middle_items: Vec<_> = item_iter.collect();

    let total_len = items.len();
    let do_ellipsize = ellipsize.map(|limit| total_len > limit).unwrap_or(false);

    let middle_to_render = if do_ellipsize {
        if let Some(limit) = ellipsize {
            middle_items
                .iter()
                .rev()
                .take(limit.saturating_sub(1))
                .rev()
                .copied()
                .collect()
        } else {
            middle_items
        }
    } else {
        middle_items
    };

    Ok(view! {
        breadcrumb(
            breadcrumb_list(
                if let Some(taxon) = root {
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! { href=(link_fn(taxon)) },
                            (&taxon.complete_name)
                        )
                    )
                }
                if do_ellipsize {
                    breadcrumb_separator()
                    breadcrumb_ellipsis()
                }
                for taxon in middle_to_render {
                    breadcrumb_separator()
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! { href=(link_fn(taxon)) },
                            (&taxon.complete_name)
                        )
                    )
                }

                if let Some(taxon) = last {
                    if root.is_some() {
                        breadcrumb_separator()
                    }
                    breadcrumb_item(
                        if link_final {
                            breadcrumb_link(
                                attrs: attributes! { href=(link_fn(taxon)) },
                                (&taxon.complete_name)
                            )
                        } else {
                            breadcrumb_page(
                                (&taxon.complete_name)
                                badge(
                                    variant: BadgeVariant::Secondary,
                                    attrs: attributes! { class="mx-3" },
                                    (taxon.rank.to_string())
                                )
                            )
                        }
                    )
                }
                (child)
            )
        )
    })
}

enum PaginationItemType {
    Previous(usize),
    Next(usize),
    Ellipsis,
    Page(usize),
}

fn pagination_pages(state: &PageState, context: Option<usize>) -> Vec<PaginationItemType> {
    let context = context.unwrap_or(2);
    let mut items = Vec::with_capacity(context * 2 + 5);
    let cur = state.current_page();
    let first = cur.saturating_sub(context).max(1);
    let last = (cur + context).min(state.total_pages());
    if cur > 1 {
        items.push(PaginationItemType::Previous(cur - 1));
    }
    if first > 1 {
        items.push(PaginationItemType::Page(1));
    }
    if first > 2 {
        items.push(PaginationItemType::Ellipsis)
    }
    for n in first..=last {
        items.push(PaginationItemType::Page(n));
    }
    if last + 1 < state.total_pages() {
        items.push(PaginationItemType::Ellipsis)
    }
    if last < state.total_pages() {
        items.push(PaginationItemType::Page(state.total_pages()));
    }
    if cur < last {
        items.push(PaginationItemType::Next(cur + 1));
    }
    items
}

#[component]
pub async fn pagination_control<T: ModifyOffset + Clone + Sync + Send>(
    state: &PageState,
    params: T,
    #[default(Some(2))] context: Option<usize>,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    let items = pagination_pages(state, context);
    let cur = state.current_page();
    Ok(view! {
        <div class=(class!(attrs.remove("class"))) (attrs)>
            pagination(
                pagination_content(
                    for item in items {
                        pagination_item(
                            match item {
                                PaginationItemType::Ellipsis => {
                                    pagination_ellipsis()
                                }
                                PaginationItemType::Page(n) => {
                                    pagination_link(
                                        active: n == cur,
                                        attrs: attributes! {
                                            href=(state.query_with_offset(
                                                state.offset_for_page(n).unwrap_or_default(),
                                                params.clone(),
                                            ))
                                        },
                                        (n.to_string())
                                    )
                                }
                                PaginationItemType::Next(n) => {
                                    pagination_next(
                                        attrs: attributes! {
                                            href=(state.query_with_offset(
                                                state.offset_for_page(n).unwrap_or_default(),
                                                params.clone(),
                                            ))
                                        }
                                    )
                                }
                                PaginationItemType::Previous(n) => {
                                    pagination_previous(
                                        attrs: attributes! {
                                            href=(state.query_with_offset(
                                                state.offset_for_page(n).unwrap_or_default(),
                                                params.clone(),
                                            ))
                                        }
                                    )
                                }
                            }
                        )
                    }
                )
            )
        </div>
    })
}

#[component]
pub async fn week_navigator(
    date: jiff::civil::Date,
    #[default] attrs: Attributes,
) -> topcoat::Result<impl View> {
    let fmt = |dt: jiff::civil::Date| dt.strftime("%b %d").to_string();
    let prev_date = date - 7.days();
    let prev_link = format!("?date={}", prev_date);
    let next_date = date + 7.days();
    let next_link = format!("?date={}", next_date);
    Ok(view! {
        pagination(
            attrs: attrs,
            pagination_content(
                pagination_item(
                    pagination_previous(attrs: attributes! { href=(prev_link) })
                )
                pagination_item(pagination_link(active: true, (fmt(date))))
                pagination_item(
                    pagination_next(attrs: attributes! { href=(next_link) })
                )
            )
        )
    })
}

#[component]
pub async fn user_menu(cx: &Cx) -> topcoat::Result<impl View> {
    let uri = uri(cx);
    let login_href = href!(login);
    let menu_open = signal(cx, || false);
    Ok(view! {
        if !login_href.is_current(cx) {
            <li class="ml-auto">
                if let Some(user) = current_user(cx).await {
                    <form
                        method="POST"
                        action=(href!(do_logout))
                        id="logoutForm"
                        class="hidden"
                    ></form>
                    dropdown_menu(
                        dropdown_menu_trigger(
                            attrs: attributes! { class="flex" },
                            avatar(
                                attrs: attributes! { class="me-2 bg-background/60" },
                                size: AvatarSize::Sm,
                                avatar_fallback(icon(data: mdi::ACCOUNT))
                            )
                            <span class="hidden md:inline-block">(&user.username)</span>
                        )
                        dropdown_menu_content(
                            alignment: DropdownMenuAlignment::Right,
                            dropdown_menu_item(
                                attrs: attributes! { type="submit" class="text-destructive" form="logoutForm" },
                                "Log Out"
                            )
                        )
                    )
                } else {
                    <a
                        class="flex"
                        href=(login_href.query(
                            LoginFormParams { redirect: Some(uri.to_string()) },
                        ))
                    >
                        "Log in"
                    </a>
                }
            </li>
        }
    })
}

const LEAFLET_INIT_SCRIPT: Asset = asset!("assets/leaflet-initialize.js");

#[component]
pub async fn leaflet_map(
    geometry: &geojson::Geometry,
    #[default] attrs: Attributes,
) -> topcoat::Result<impl View> {
    let id = Uuid::new_v4().to_string();
    Ok(view! {
        <div id=(&id) (attrs)></div>
        <script src=(LEAFLET_INIT_SCRIPT)></script>
        <script>
            (format!("var geojson = {};", geometry))
            (format!("var id = '{}';", id))
            "initializeLeaflet(id, geojson);"
        </script>
    })
}

#[component]
pub async fn taxa_grid(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div class=(class!("results-grid", attrs.remove("class"))) (attrs)>(child)</div>
    })
}
#[component]
pub async fn taxa_grid_item(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class!(
                "p-6 h-full flex flex-col items-center text-center justify-center",
                attrs.remove("class"),
            ))
            (attrs)
        >
            (child)
        </div>
    })
}

#[component]
pub async fn taxon_icon(
    url: Option<&String>,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    Ok(view! {
        if let Some(photo) = url.as_ref() {
            <img
                class=(class!("block rounded-xl border", attrs.remove("class")))
                (attrs)
                src=(photo)
            >
        } else {
            icon(data: mdi::LEAF_CIRCLE, size: 75, label: "Missing Image")
        }
    })
}

#[shard]
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
                            <a href=(href!(taxa::details, TaxonId(taxon.id)))>
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
                        class="text-foreground hover:opacity-80 focus-within:opacity-80 opacity-50"
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
