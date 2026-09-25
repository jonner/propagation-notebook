use topcoat::{
    context::Cx,
    router::href,
    view::{Attributes, Child, View, class, component, view},
};

use libpropagation::{
    region::{Region, RegionalHarvestWindow},
    taxonomy::Taxon,
};

use crate::{
    components::pn::{conservation_status_badge, origin_badge},
    regions,
    taxa::{self, RegionHarvestWindowSummary},
};

/// Renders a 52-week harvest window timeline component with week blocks, active
/// harvest window highlight and a vertical line showing the current day/week.
#[component]
pub async fn harvest_timeline(
    timeline: &RegionalHarvestWindow,
    #[default] current_doy: Option<i16>,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    let cdoy = current_doy.unwrap_or_else(|| jiff::Zoned::now().date().day_of_year());
    let inactive_class = timeline.is_empty().then_some("inactive");

    // Compute left offset percentage for current date marker ignoring leap days
    let marker_left_pct = (f32::from(cdoy - 1) / 365.0) * 100.0;

    Ok(view! {
        <div
            class=(class!(
                "relative flex items-center gap-0 items-stretch w-full h-full select-none min-h-[1em]",
                inactive_class,
                attrs.remove("class"),
            ))
            (attrs)
        >
            for w in 1..=52 {
                {
                    let in_window = if let (Some(start_week), Some(end_week)) = (
                        timeline.start_week(),
                        timeline.end_week(),
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
    })
}

// NOTE: This function assumes that taxa is a list of taxa that have only a
//  single regional_status: the region we're displaying the table for
#[component]
pub async fn regional_taxa_table(
    cx: &Cx,
    taxa: &[Taxon],
    #[default] current_doy: Option<i16>,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        harvest_table(
            attrs: attrs,
            for taxon in taxa {
                if let Some(timeline) = taxon.regional_statuses.get().first() {
                    harvest_table_row(
                        harvest_table_row_header(
                            <span class="latin">
                                <a href=(href!(taxa::details, taxa::TaxonId(taxon.id)))>
                                    (&taxon.complete_name)
                                </a>
                            </span>
                            <div class="flex items-center gap-4">
                                if let Some(origin) = timeline.origin {
                                    origin_badge(origin: origin)
                                }
                                if let Some(status) = timeline.conservation_status {
                                    conservation_status_badge(status: status)
                                }
                            </div>
                        )
                        harvest_table_row_timeline(
                            timeline: &timeline.harvest_window,
                            current_doy: current_doy
                        )
                    )
                }
            }
            (child)
        )
    })
}

#[component]
pub async fn taxon_regional_table(
    cx: &Cx,
    regions: &[(Region, RegionHarvestWindowSummary)],
    #[default] current_doy: Option<i16>,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        if regions.is_empty() {
            <div class="text-muted-foreground">"None"</div>
        } else {
            harvest_table(
                attrs: attrs,
                for (region, timeline) in regions {
                    harvest_table_row(
                        harvest_table_row_header(
                            <a
                                href=(href!(regions::overview, regions::RegionId(region.id)))
                            >
                                (&region.name)
                            </a>
                            <div class="flex items-center gap-4">
                                if let Some(origin) = timeline.origin {
                                    origin_badge(origin: origin)
                                }
                                if let Some(status) = timeline.conservation_status {
                                    conservation_status_badge(status: status)
                                }
                            </div>
                        )
                        harvest_table_row_timeline(
                            timeline: &timeline.harvest_window,
                            current_doy: current_doy
                        )
                    )
                }
                (child)
            )
        }
    })
}

#[component]
pub async fn harvest_table_row(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class!("flex flex-col gap-1 md:contents", attrs.remove("class")))
            (attrs)
        >
            (child)
        </div>
    })
}

#[component]
pub async fn harvest_table_row_header(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class!("flex gap-3 items-center w-full", attrs.remove("class")))
            (attrs)
        >
            (child)
        </div>
    })
}

#[component]
pub async fn harvest_table_row_timeline(
    timeline: &RegionalHarvestWindow,
    #[default] current_doy: Option<i16>,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class!("flex h-full items-center gap-x-6", attrs.remove("class")))
            (attrs)
        >
            <div class="h-full w-120">
                harvest_timeline(timeline: timeline, current_doy: current_doy)
            </div>
            <div class="text-nowrap hidden md:block">
                if timeline.start_doy.is_some() && timeline.end_doy.is_some() {
                    (timeline.to_string())
                }
            </div>
        </div>
    })
}

#[component]
pub async fn harvest_table(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class!(
                "flex flex-col gap-3 md:grid md:grid-cols-[max-content_auto] md:gap-x-6 md:gap-y-2 md:items-center",
                attrs.remove("class"),
            ))
            (attrs)
        >
            (child)
        </div>
    })
}
