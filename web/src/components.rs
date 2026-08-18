use jiff::ToSpan;
use libpropagation::{
    citation::Citation,
    propagation::PropagationProcedure,
    region::{ConservationStatus, Origin, RegionalHarvestWindow, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use topcoat::{
    context::Cx,
    icon::icon,
    router::href,
    view::{Attributes, View, attributes, class, component, view},
};

use crate::{
    leaflet::Map,
    mdi,
    util::{ModifyOffset, PageState},
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
    #[default(2)] context: usize,
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
        <nav class="flex gap-3 items-center my-4">
            <ul class="contents">
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

/// Renders a 52-week harvest window timeline component with week blocks, active
/// harvest window highlight and a vertical line showing the current day/week.
#[component]
pub async fn harvest_timeline(
    window: &RegionalHarvestWindow,
    #[default] current_doy: Option<i16>,
    #[default] attrs: Attributes,
) -> topcoat::Result {
    let cdoy = current_doy.unwrap_or_else(|| jiff::Zoned::now().date().day_of_year());
    let inactive_class = window.is_empty().then_some("inactive");

    // Compute left offset percentage for current date marker ignoring leap days
    let marker_left_pct = (f32::from(cdoy - 1) / 365.0) * 100.0;

    view! {
        <div
            class=(class!(
                "relative flex items-center gap-0 items-stretch w-full h-full select-none min-h-[1em]",
                inactive_class,
            ))
            (attrs)
        >
            for w in 1..=52 {
                {
                    let in_window = if let (Some(start_week), Some(end_week)) = (
                        window.start_week(),
                        window.end_week(),
                    ) {
                        if start_week <= end_week {
                            w >= start_week && w <= end_week
                        } else {
                            w >= start_week || w <= end_week
                        }
                    } else {
                        false
                    };

                    let bg_class = if in_window { "bg-leaf/50" } else { "bg-brown/20" };

                    <div class=(class!("flex-grow", bg_class))></div>
                }
            }
            // Current date vertical indicator
            <div
                class="absolute top-0 bottom-0 w-[2px] bg-mallard z-20"
                style=(format!("left: {:.2}%;", marker_left_pct))
            ></div>
        </div>
    }
}

#[component]
pub async fn origin_badge(origin: Origin, #[default] mut attrs: Attributes) -> topcoat::Result {
    let vals = match origin {
        Origin::Introduced => Some(("introduced", "Introduced")),
        Origin::Unknown => Some(("unknown", "Unknown origin")),
        Origin::Native => None,
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
    #[default] current_doy: Option<i16>,
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
    view! {
        harvest_table(
            items: &items,
            current_doy: current_doy,
            attrs: attrs,
            child: child
        )
    }
}

#[component]
pub async fn taxon_regional_table(
    regions: &[RegionalTaxonStatus],
    #[default] current_doy: Option<i16>,
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
    view! {
        harvest_table(
            items: &items,
            current_doy: current_doy,
            attrs: attrs,
            child: child
        )
    }
}

#[component]
pub async fn harvest_table(
    items: &[(&str, String, &RegionalTaxonStatus)],
    #[default] current_doy: Option<i16>,
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> topcoat::Result {
    view! {
        <div
            class=(class!(
                "flex flex-col gap-3 md:grid md:grid-cols-[max-content_auto] md:gap-x-6 md:gap-y-2 md:items-center",
                attrs.remove("class"),
            ))
            (attrs)
        >
            for item in items {
                let name = item.0;
                let path = &item.1;
                let rts = item.2;
                <div class="flex flex-col gap-1 md:contents">
                    <div class="flex gap-3 items-center w-full">
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
                    <div class="flex h-full items-center gap-x-6">
                        <div class="h-full w-120">
                            harvest_timeline(
                                window: &rts.harvest_window,
                                current_doy: current_doy
                            )
                        </div>
                        <div class="text-nowrap hidden md:block">
                            if rts.harvest_window.start_doy.is_some()
                                && rts.harvest_window.end_doy.is_some() {
                                (rts.harvest_window.to_string())
                            }
                        </div>
                    </div>
                </div>
            }
            <div class="md:col-span-2">(child)</div>
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
    let mut breadcrumb_iter = items.into_iter();
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

#[component]
pub async fn leaflet_map(
    geometry: &geojson::Geometry,
    #[default] attrs: Attributes,
) -> topcoat::Result {
    let leaflet_script = Map::new(geometry);
    view! {
        <div id=(&leaflet_script.id) (attrs)></div>
        (leaflet_script)
    }
}

#[component]
pub async fn week_navigator(date: jiff::civil::Date) -> topcoat::Result {
    let prev_date = date - 7.days();
    let prev_link = format!("?date={}", prev_date);
    let next_date = date + 7.days();
    let next_link = format!("?date={}", next_date);
    let date_str = date.strftime("%B %d").to_string();
    view! {
        <nav class="flex gap-4">
            <ol class="contents">
                <li><a href=(prev_link)>"Prev"</a></li>
                <li>(date_str)</li>
                <li><a href=(next_link)>"Next"</a></li>
            </ol>
        </nav>
    }
}
