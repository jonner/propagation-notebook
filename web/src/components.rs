use libpropagation::{
    citation::Citation,
    propagation::PropagationProcedure,
    region::{ConservationStatus, Origin, RegionalHarvestWindow, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use topcoat::{
    icon::icon,
    view::{Attributes, View, attributes, class, component, view},
};

use crate::{
    mdi,
    util::{ModifyOffset, PageState, Path},
};

#[component]
pub async fn citation_list(citations: Vec<&Citation>) -> topcoat::Result {
    view! {
        <ul>
            for citation in citations {
                <li>
                    <a href=(format!("/citation/{}", citation.id))>(&citation.title)</a>
                </li>
            }
        </ul>
    }
}

enum PageLinkType {
    Ellipsis,
    Page(usize),
    Icon(usize, View),
}

#[component]
pub async fn pagination_control<'p, T: ModifyOffset + Clone + Sync + Send + 'p>(
    state: &PageState,
    params: &'p T,
    #[default(3)] context: usize,
) -> topcoat::Result {
    let mut links = Vec::with_capacity(context * 2 + 5);
    let cur = state.current_page();
    let first = cur.saturating_sub(context).max(1);
    let last = (cur + context).min(state.total_pages());
    if cur > 1 {
        links.push(PageLinkType::Icon(
            cur - 1,
            view! {
                icon(
                    data: mdi::NAVIGATE_BEFORE,
                    label: "Previous",
                    attrs: attributes! { class="icon" }
                )
            }
            .unwrap(),
        ));
    }
    if first > 1 {
        links.push(PageLinkType::Page(1));
    }
    if first > 2 {
        links.push(PageLinkType::Ellipsis)
    }
    for n in first..=last {
        links.push(PageLinkType::Page(n));
    }
    if last < (state.total_pages() - 1) {
        links.push(PageLinkType::Ellipsis)
    }
    if last < state.total_pages() {
        links.push(PageLinkType::Page(state.total_pages()));
    }
    if cur < last {
        links.push(PageLinkType::Icon(
            cur + 1,
            view! {
                icon(
                    data: mdi::NAVIGATE_NEXT,
                    label: "Next",
                    attrs: attributes! { class="icon" }
                )
            }
            .unwrap(),
        ));
    }
    view! {
        <nav class="mt-4">
            <ul class="flex gap-3 items-center">
                for item in links {
                    <li>
                        match item {
                            PageLinkType::Ellipsis => {
                                icon(
                                    data: mdi::ELLIPSIS_HORIZONTAL,
                                    label: "skipped",
                                    attrs: attributes! { class="icon" }
                                )
                            }
                            PageLinkType::Page(n) => {
                                if n == state.current_page() {
                                    <span class="inline-block font-bold self-center">
                                        (n.to_string())
                                    </span>
                                } else {
                                    <a
                                        href=(state
                                            .query_with_offset(
                                                state.offset_for_page(n).unwrap_or_default(),
                                                params.clone(),
                                            ))
                                    >
                                        (n.to_string())
                                    </a>
                                }
                            }
                            PageLinkType::Icon(n, view) => {
                                <a
                                    class="button"
                                    href=(state
                                        .query_with_offset(
                                            state.offset_for_page(n).unwrap_or_default(),
                                            params.clone(),
                                        ))
                                >
                                    (view)
                                </a>
                            }
                        }
                    </li>
                }
            </ul>
        </nav>
    }
}

#[component]
pub async fn propagation_details(procedure: &PropagationProcedure) -> topcoat::Result {
    view! {
        <h3>"Type"</h3>
        <div>(procedure.r#type.to_string())</div>
        <h3>"Notes"</h3>
        <div>(procedure.notes.as_deref().unwrap_or_default())</div>
        <h3>"Instructions"</h3>
        <div>(&procedure.instructions)</div>
        <h3>"Citations"</h3>
        <div>
            if !procedure.citations.get().is_empty() {
                citation_list(citations: procedure.citations.get().iter().collect())
            } else {
                "None"
            }
        </div>
    }
}

/// Renders a 52-week harvest window timeline component with week blocks,
/// active harvest window highlight (and optional peak window), month headers,
/// and a vertical line showing the current day/week.
#[component]
pub async fn harvest_timeline(
    window: &RegionalHarvestWindow,
    #[default] current_week: Option<i16>,
    #[default] attrs: Attributes,
) -> topcoat::Result {
    let (Some(start_week), Some(end_week)) = (window.start_week(), window.end_week()) else {
        return view! {};
    };

    let cw =
        current_week.unwrap_or_else(|| jiff::Zoned::now().date().iso_week_date().week().into());

    // Compute left offset percentage for current week marker
    let marker_left_pct = ((cw as f32 - 0.5) / 52.0) * 100.0;

    view! {
        <div class="relative w-full select-none text-xs font-sans" (attrs)>
            <div class="relative w-full">
                // <!-- 52 Week Blocks Grid -->
                <div
                    class="grid grid-cols-[repeat(52,minmax(0,1fr))] gap-[1px] min-h-[1em]"
                >
                    for w in 1..=52 {
                        {
                            let in_window = if start_week <= end_week {
                                w >= start_week && w <= end_week
                            } else {
                                w >= start_week || w <= end_week
                            };

                            let bg_class = if in_window {
                                "bg-leaf/50 border border-leaf/60"
                            } else {
                                "bg-brown/20 border border-brown/22"
                            };

                            <div class=(format!("h-full rounded-[2px] {}", bg_class))></div>
                        }
                    }
                </div>

                // <!-- Current Week Vertical Indicator Marker -->
                <div
                    class="absolute -top-1 -bottom-1 w-[2px] bg-brown z-20"
                    style=(format!("left: {:.2}%;", marker_left_pct))
                ></div>
            </div>
        </div>
    }
}

#[component]
pub async fn origin_badge(origin: Origin, #[default] mut attrs: Attributes) -> topcoat::Result {
    let vals = match origin {
        Origin::Introduced => Some(("introduced", "Introduced")),
        Origin::Native => None,
        _ => None,
    };
    if let Some((klass, text)) = vals {
        view! {
            <div
                class=(class!(klass, "badge", attrs.remove("class")))
                (attrs)
                title=(origin.to_string())
            >
                (text)
            </div>
        }
    } else {
        view! {}
    }
}

#[component]
pub async fn conservation_status_badge(
    status: ConservationStatus,
    #[default] mut attrs: Attributes,
) -> topcoat::Result {
    let (klass, text) = match status {
        ConservationStatus::Endangered => ("endangered", "EN"),
        ConservationStatus::Threatened => ("threatened", "TH"),
        ConservationStatus::SpecialConcern => ("specialconcern", "SC"),
    };
    view! {
        <div
            class=(class!(klass, "badge", attrs.remove("class")))
            (attrs)
            title=(status.to_string())
        >
            (text)
        </div>
    }
}

#[component]
pub async fn regional_taxa_table(
    taxa: &[Taxon],
    #[default] attrs: Attributes,
    #[default] child: View,
) -> topcoat::Result {
    let items: Vec<_> = taxa
        .iter()
        .filter_map(|taxon| {
            taxon
                .regional_statuses
                .get()
                .first()
                .map(|rts| (taxon.complete_name.as_str(), taxon.path(), rts))
        })
        .collect();
    view! { harvest_table(items: &items, attrs: attrs, child: child) }
}

#[component]
pub async fn taxon_regional_table(
    regions: &[RegionalTaxonStatus],
    #[default] attrs: Attributes,
    #[default] child: View,
) -> topcoat::Result {
    let items: Vec<_> = regions
        .iter()
        .map(|rts| {
            let region = rts.region.get();
            (region.name.as_str(), region.path(), rts)
        })
        .collect();
    view! { harvest_table(items: &items, attrs: attrs, child: child) }
}

#[component]
pub async fn harvest_table(
    items: &[(&str, String, &RegionalTaxonStatus)],
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> topcoat::Result {
    view! {
        <div
            class=(class!("flex flex-col gap-6 md:gap-2", attrs.remove("class")))
            (attrs)
        >
            for item in items {
                let name = item.0;
                let path = &item.1;
                let rts = item.2;
                <div class="flex flex-col md:flex-row md:gap-4 w-full">
                    <div class="flex gap-4 items-center w-full md:w-1/4">
                        <span class="latin"><a href=(path)>(name)</a></span>
                        <div class="flex items-center gap-4">
                            if let Some(origin) = rts.origin {
                                origin_badge(origin: origin)
                            }
                            if let Some(status) = rts.conservation_status {
                                conservation_status_badge(status: status)
                            }
                        </div>
                    </div>
                    if rts.harvest_window.start_doy.is_some()
                        && rts.harvest_window.end_doy.is_some() {
                        <div class="flex items-center gap-x-6">
                            <div class="w-120">
                                harvest_timeline(window: &rts.harvest_window)
                            </div>
                            <div class="text-nowrap">
                                (rts.harvest_window.to_string())
                            </div>
                        </div>
                    }
                </div>
            }
            (child)
        </div>
    }
}

pub struct Breadcrumb {
    pub url: Option<String>,
    pub text: String,
}

#[component]
pub async fn breadcrumbs(
    items: Vec<Breadcrumb>,
    #[default] ellipsize: Option<usize>,
) -> topcoat::Result {
    let do_ellipsize = ellipsize.map(|n| items.len() > n + 1) == Some(true);
    let mut breadcrumb_iter = items.into_iter().rev();
    let Some(root) = breadcrumb_iter.next() else {
        return view! {};
    };
    let rest: Vec<_> = if let Some(ellipsize) = ellipsize {
        breadcrumb_iter.rev().take(ellipsize).rev().collect()
    } else {
        breadcrumb_iter.collect()
    };
    view! {
        <nav class="breadcrumbs">
            <ol>
                <li><a href=(root.url)>(&root.text)</a></li>
                if do_ellipsize {
                    <li>
                        icon(
                            data: mdi::NAVIGATE_NEXT,
                            label: "separator",
                            attrs: attributes! { class="icon" }
                        )
                        <a>
                            icon(
                                data: mdi::ELLIPSIS_HORIZONTAL,
                                label: "skipped",
                                attrs: attributes! { class="icon" }
                            )
                        </a>
                    </li>
                }
                for item in rest {
                    <li>
                        icon(
                            data: mdi::NAVIGATE_NEXT,
                            label: "separator",
                            attrs: attributes! { class="icon" }
                        )
                        <a href=(item.url)>(&item.text)</a>
                    </li>
                }
            </ol>
        </nav>
    }
}
