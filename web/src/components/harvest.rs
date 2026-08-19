use topcoat::{
    context::Cx,
    router::href,
    view::{Attributes, View, class, component, view},
};

use libpropagation::{
    region::{RegionalHarvestWindow, RegionalTaxonStatus},
    taxonomy::Taxon,
};

use crate::{
    components::badge::{conservation_status_badge, origin_badge},
    regions, taxa,
};

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
pub async fn regional_taxa_table(
    cx: &Cx,
    taxa: &[Taxon],
    #[default] current_doy: Option<i16>,
    #[default] attrs: Attributes,
    #[default] child: View,
) -> topcoat::Result {
    let items: Vec<_> = taxa
        .iter()
        .filter_map(|taxon| {
            taxon.regional_statuses.get().first().map(|rts| {
                (
                    taxon.complete_name.as_str(),
                    href!(taxa::details, taxa::TaxonId(taxon.id)).resolve(cx),
                    rts,
                )
            })
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
    cx: &Cx,
    regions: &[RegionalTaxonStatus],
    #[default] current_doy: Option<i16>,
    #[default] attrs: Attributes,
    #[default] child: View,
) -> topcoat::Result {
    let items: Vec<_> = regions
        .iter()
        .map(|rts| {
            let region = rts.region.get();
            (
                region.name.as_str(),
                href!(regions::overview, regions::RegionId(region.id)).resolve(cx),
                rts,
            )
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
