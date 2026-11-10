use libpropagation::region::{ConservationStatus, Origin};
use topcoat::{
    context::Cx,
    icon::icon,
    view::{Attributes, Child, View, ViewExt, attributes, class, component, view},
};

use crate::{
    assets::mdi,
    components::{pagination::*, tooltip::*},
    util::{PageState, Query},
};

#[component]
pub async fn origin_badge(
    cx: &Cx,
    origin: Origin,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    let vals = match origin {
        Origin::Introduced => Some(("introduced", "IN", "Introduced")),
        Origin::Unknown => Some(("unknown", "UN", "Unknown or mixed origin")),
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
    cx: &Cx,
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
pub async fn pagination_control(
    cx: &Cx,
    state: &PageState,
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
                                            href=(Query::from_cx(cx).with(
                                                "offset",
                                                state.offset_for_page(n).unwrap_or_default(),
                                            ))
                                        },
                                        (n.to_string())
                                    )
                                }
                                PaginationItemType::Next(n) => {
                                    pagination_next(
                                        attrs: attributes! {
                                            href=(Query::from_cx(cx).with(
                                                "offset",
                                                state.offset_for_page(n).unwrap_or_default(),
                                            ))
                                        }
                                    )
                                }
                                PaginationItemType::Previous(n) => {
                                    pagination_previous(
                                        attrs: attributes! {
                                            href=(Query::from_cx(cx).with(
                                                "offset",
                                                state.offset_for_page(n).unwrap_or_default(),
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
pub async fn taxon_icon(
    cx: &Cx,
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

#[component]
pub async fn item_term(
    cx: &Cx,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class!("text-muted-foreground text-sm", attrs.remove("class")))
            (attrs)
        >
            (child)
        </div>
    })
}
#[component]
pub async fn item_value(
    cx: &Cx,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! { <div (attrs)>(child)</div> })
}
