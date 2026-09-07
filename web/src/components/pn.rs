use jiff::ToSpan;
use libpropagation::{
    region::{ConservationStatus, Origin},
    taxonomy::Taxon,
};
use topcoat::{
    context::Cx,
    router::{href, query_params},
    view::{Attributes, View, ViewExt, attributes, class, component, view},
};

use crate::{
    components::{badge::*, breadcrumb::*, pagination::*, tooltip::*},
    taxa::TaxaListParams,
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
pub async fn ancestor_breadcrumbs(
    cx: &Cx,
    items: &[&Taxon],
    #[default] link_final: bool,
    #[default(Some(2))] ellipsize: Option<usize>,
) -> topcoat::Result<impl View> {
    let params = query_params::<TaxaListParams>(cx)
        .cloned()
        .unwrap_or_default();
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
                            attrs: attributes! {
                                href=(href!(crate::taxa::taxonomy).query(
                                    TaxaListParams {
                                        offset: None,
                                        parent: Some(taxon.id),
                                        fmt: params.fmt,
                                        region: params.region,
                                    },
                                ))
                            },
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
                            attrs: attributes! {
                                href=(href!(crate::taxa::taxonomy).query(
                                    TaxaListParams {
                                        offset: None,
                                        parent: Some(taxon.id),
                                        fmt: params.fmt,
                                        region: params.region,
                                    },
                                ))
                            },
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
                                attrs: attributes! {
                                    href=(href!(crate::taxa::taxonomy).query(
                                        TaxaListParams {
                                            offset: None,
                                            parent: Some(taxon.id),
                                            fmt: params.fmt,
                                            region: params.region,
                                        },
                                    ))
                                },
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
