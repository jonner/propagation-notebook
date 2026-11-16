use libpropagation::{
    region::{HarvestFilter, OriginFilter, Region, RegionCategory},
    taxonomy::Taxon,
};
use topcoat::{
    context::{Cx, memoize},
    icon::{IconData, icon},
    metadata::Metadata,
    router::{href, page, query_params},
    view::{View, ViewExt, attributes, component, element_id, error_boundary, view},
};

use crate::{
    assets::mdi,
    components::{
        button::{ButtonSize, ButtonVariant, button, button_variants},
        card::{card, card_content, card_description, card_footer, card_header, card_title},
        harvest::harvest_timeline,
        label::label,
        pn::{conservation_status_badge, origin_badge, taxon_icon},
        search::taxon_search_bar,
        select::select,
    },
    context::db,
    regions, taxa,
};

#[derive(Debug, Clone)]
#[query_params(error = bad_request)]
struct HomeParams {
    region: Option<u64>,
}
const TAGLINE: &str = "Seed-collection and propagation information for native plants.";

/// The number of species cards shown in the seasonal panel.
const SEASONAL_LIMIT: usize = 6;

#[memoize(as_ref)]
async fn regions(cx: &Cx) -> topcoat::Result<Vec<Region>> {
    Region::all()
        .order_by(Region::fields().name().asc())
        .exec(&mut db(cx))
        .await
        .map_err(Into::into)
}

#[page("/", with(Metadata::new().description(TAGLINE)))]
pub async fn index(cx: &Cx) -> topcoat::Result<impl View> {
    let regions = regions(cx).await?;
    let selected = default_region(cx).await?.cloned();
    let top_level_regions = regions
        .iter()
        .filter(|region| matches!(region.category, RegionCategory::Province))
        .chain(
            regions
                .iter()
                .filter(|region| matches!(region.category, RegionCategory::Nation)),
        )
        .map(|region| (region.id, region.name.clone()))
        .collect::<Vec<_>>();

    Ok(view! {
        <div class="flex flex-col gap-15 md:gap-20 pb-6">
            card(
                attrs: attributes! { class="max-w-4xl mx-auto -mt-8 md:-mt-12 shadow-sm" },
                card_content(
                    attrs: attributes! { class="flex flex-col gap-5 md:py-10! md:px-20!" },
                    <div class="flex flex-col gap-2">
                        <h1>"Propagation Notebook"</h1>
                        <p class="text-lg text-muted-foreground">(TAGLINE)</p>
                    </div>
                    taxon_search_bar(show_button: true)
                    <nav class="flex flex-wrap gap-x-5 gap-y-2 text-sm">
                        <a href=(href!(taxa::explore))>"Browse taxonomy"</a>
                        <a href=(href!(regions::list))>"Browse regions"</a>
                        <a href=(href!(crate::about))>"About this site"</a>
                    </nav>
                )
            )

            <section class="flex flex-col gap-5">
                <div class="flex flex-col gap-4">
                    <div class="flex flex-wrap items-baseline-last gap-x-3 gap-y-2">
                        <h2 class="mb-0">"In season near you"</h2>
                        if let Some(region) = selected.as_ref() {
                            <form
                                method="get"
                                action=(href!(index))
                                class="flex items-end gap-2"
                            >
                                <div class="flex flex-col gap-1">
                                    let region_select_id = element_id(cx);
                                    label(
                                        attrs: attributes! { class="text-muted-foreground" for=(&region_select_id) },
                                        "Region"
                                    )
                                    select(
                                        attrs: attributes! { name="region" id=(&region_select_id) class="min-w-48" },
                                        if !top_level_regions
                                            .iter()
                                            .any(|(id, _)| *id == region.id) {
                                            <option value=(region.id) selected=(true)>
                                                (&region.name)
                                            </option>
                                        }
                                        for (option_id, option_name) in top_level_regions {
                                            <option
                                                value=(option_id)
                                                selected=(option_id == region.id)
                                            >
                                                (&option_name)
                                            </option>
                                        }
                                    )
                                </div>
                                button(
                                    variant: ButtonVariant::Outline,
                                    attrs: attributes! { type="submit" },
                                    "Update"
                                )
                            </form>
                        }
                        <a
                            class="ml-auto text-sm inline-flex items-baseline gap-2"
                            href=(href!(regions::list))
                        >
                            "Browse all regions"
                            icon(data: mdi::CHEVRON_DOUBLE_RIGHT)
                        </a>
                    </div>
                    if let Some(region) = selected.as_ref() {
                        <p class="text-sm text-muted-foreground">
                            "Native species bearing seed in "
                            <span>(&region.name)</span>
                            " right now, based on iNaturalist observations."
                        </p>
                    }
                </div>

                if let Some(region) = selected.as_ref() {
                    error_boundary(
                        fallback: |_error| {
                            Ok(
                                view! {
                                    card(
                                        card_content(
                                            <p>
                                                "Seasonal data isn't available right now. "
                                                <a href=(href!(regions::list))>"Browse all regions →"</a>
                                            </p>
                                        )
                                    )
                                }.boxed(

                                ),
                            )
                        },
                        seasonal_data(region: region.clone())
                    )
                } else {
                    card(
                        card_header(card_title("Regions are being added"))
                        card_content(
                            card_description(
                                "Seasonal statuses are not available for any region yet. \
                                You can still browse the taxonomy or read about the project."
                            )
                        )
                        card_footer(
                            <a
                                class=(button_variants(
                                    ButtonVariant::Outline,
                                    ButtonSize::Sm,
                                ))
                                href=(href!(taxa::explore))
                            >
                                "Browse taxonomy"
                            </a>
                            <a
                                class=(button_variants(
                                    ButtonVariant::Outline,
                                    ButtonSize::Sm,
                                ))
                                href=(href!(crate::about))
                            >
                                "About this site"
                            </a>
                        )
                    )
                }
            </section>
            // explore_section()
            // <section class="flex flex-col gap-2 pt-6 text-sm text-muted-foreground">
            //     <h2>"About the notebook"</h2>
            //     <p>
            //         "The goal of this site is to become a comprehensive reference for \
            //         collecting seeds and propagating native plants for ecological restoration."
            //     </p>
            //     <p>
            //         "Taxonomy from "
            //         <a href="https://www.itis.gov">"ITIS"</a>
            //         " · Phenology from "
            //         <a href="https://inaturalist.org">"iNaturalist"</a>
            //     </p>
            //     <p><a href=(href!(crate::about))>"Read the mission →"</a></p>
            // </section>
        </div>
    })
}

/// Picks the region shown on the landing page.
///
/// An explicit `?region=` wins, then the first
/// province, the first nation, and finally any region.
async fn default_region(cx: &Cx) -> topcoat::Result<Option<&Region>> {
    let params = query_params::<HomeParams>(cx)?;
    let regions = regions(cx).await?;
    if let Some(id) = params.region
        && let Some(region) = regions.iter().find(|region| region.id == id)
    {
        return Ok(Some(region));
    }

    Ok(regions
        .iter()
        .find(|region| matches!(region.category, RegionCategory::Province))
        .or_else(|| {
            regions
                .iter()
                .find(|region| matches!(region.category, RegionCategory::Nation))
        })
        .or_else(|| regions.first()))
}

#[component]
async fn seasonal_data(cx: &Cx, region: Region) -> topcoat::Result<impl View> {
    let mut db = db(cx);
    let (fruiting_now, fruiting_taxa) = region
        .get_random_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::ReadyNow),
            SEASONAL_LIMIT,
            true,
        )
        .await?;
    let (winding_down, _) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::EndingSoon {
                context: 10,
                doy: None,
            }),
            None,
            Some(1),
        )
        .await?;
    let (ramping_up, _) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::StartingSoon {
                context: 10,
                doy: None,
            }),
            None,
            Some(1),
        )
        .await?;

    Ok(view! {
        <div class="flex flex-col gap-6">
            <div class="grid gap-4 sm:grid-cols-3">
                <a
                    class="block h-full no-underline text-foreground"
                    href=(href!(regions::fruiting_now, regions::region_id: region.id))
                >
                    card(
                        attrs: attributes! { class="h-full transition-colors hover:bg-foreground/5" },
                        card_content(
                            attrs: attributes! { class="flex flex-col gap-1" },
                            <div class="text-4xl font-bold text-header">
                                (fruiting_now)
                            </div>
                            <div class="text-sm text-muted-foreground">
                                "Fruiting now"
                            </div>
                        )
                    )
                </a>
                <a
                    class="block h-full no-underline text-foreground"
                    href=(href!(regions::fruiting_done, regions::region_id: region.id))
                >
                    card(
                        attrs: attributes! { class="h-full transition-colors hover:bg-foreground/5" },
                        card_content(
                            attrs: attributes! { class="flex flex-col gap-1" },
                            <div class="text-4xl font-bold text-header">
                                (winding_down)
                            </div>
                            <div class="text-sm text-muted-foreground">
                                "Winding down"
                            </div>
                        )
                    )
                </a>
                <a
                    class="block h-full no-underline text-foreground"
                    href=(href!(regions::fruiting_soon, regions::region_id: region.id))
                >
                    card(
                        attrs: attributes! { class="h-full transition-colors hover:bg-foreground/5" },
                        card_content(
                            attrs: attributes! { class="flex flex-col gap-1" },
                            <div class="text-4xl font-bold text-header">
                                (ramping_up)
                            </div>
                            <div class="text-sm text-muted-foreground">
                                "Ramping up"
                            </div>
                        )
                    )
                </a>
            </div>

            if fruiting_taxa.is_empty() {
                <p class="text-muted-foreground">
                    "No seed-window data yet for this region. "
                    <a
                        href=(href!(regions::fruiting_all, regions::region_id: region.id))
                    >
                        "See all species"
                    </a>
                    " or browse the "
                    <a href=(href!(taxa::explore))>"taxonomy"</a>
                    ". Seed windows come from iNaturalist observations."
                </p>
            } else {
                <div class="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-6">
                    for taxon in fruiting_taxa {
                        seasonal_taxon_card(taxon: taxon)
                    }
                </div>
                <a
                    class="inline-flex items-baseline gap-2"
                    href=(href!(regions::fruiting_now, regions::region_id: region.id))
                >
                    (format!(
                        "See all {} native species fruiting in {}",
                        fruiting_now,
                        region.name,
                    ))
                    icon(data: mdi::CHEVRON_DOUBLE_RIGHT)
                </a>
            }
        </div>
    })
}

#[component]
async fn seasonal_taxon_card(cx: &Cx, taxon: Taxon) -> topcoat::Result<impl View> {
    let regional_status = taxon.regional_statuses.get().first().cloned();

    Ok(view! {
        <div class="flex flex-col gap-2">
            <a
                class="no-underline"
                href=(href!(taxa::details, taxa::taxon_id: taxon.id))
            >
                taxon_icon(
                    url: taxon
                        .photo
                        .get()
                        .as_ref()
                        .and_then(
                            |photo| {
                                photo.medium_url.as_ref().or(photo.square_url.as_ref())
                            },
                        ),
                    attrs: attributes! {
                        alt=(&taxon.complete_name)
                        class="aspect-square w-full object-cover"
                    }
                )
            </a>
            <a
                class="latin text-sm truncate"
                href=(href!(taxa::details, taxa::taxon_id: taxon.id))
            >
                (&taxon.complete_name)
            </a>
            if let Some(regional_status) = regional_status {
                harvest_timeline(
                    timeline: &regional_status.harvest_window,
                    attrs: attributes! { class="h-2" }
                )
                <div class="text-xs text-muted-foreground">
                    (regional_status.harvest_window.to_string())
                </div>
                <div class="flex gap-1">
                    if let Some(origin) = regional_status.origin {
                        origin_badge(origin: origin)
                    }
                    if let Some(status) = regional_status.conservation_status {
                        conservation_status_badge(status: status)
                    }
                </div>
            }
        </div>
    })
}

#[component]
async fn explore_section(cx: &Cx) -> topcoat::Result<impl View> {
    Ok(view! {
        <section class="flex flex-col gap-6">
            <h2>"Explore the notebook"</h2>
            <div class="grid gap-6 md:grid-cols-3">
                explore_card(
                    icon_data: mdi::FORMAT_LIST_BULLETED,
                    title: "Taxonomy",
                    description: "Browse the tree, photographs, synonyms, and common names.",
                    link_label: "Browse taxonomy",
                    url: href!(taxa::explore).resolve(cx)
                )
                explore_card(
                    icon_data: mdi::GLOBE,
                    title: "Seed windows",
                    description: "See when native plants in your area are bearing seed.",
                    link_label: "Browse regions",
                    url: href!(regions::list).resolve(cx)
                )
                explore_card(
                    icon_data: mdi::LEAF_CIRCLE,
                    title: "Propagation",
                    description: "Find cleaning and germination protocols on each species page.",
                    link_label: "Explore species",
                    url: href!(taxa::explore).resolve(cx)
                )
            </div>
        </section>
    })
}

#[component]
async fn explore_card(
    cx: &Cx,
    icon_data: IconData,
    title: &str,
    description: &str,
    link_label: &str,
    url: String,
) -> topcoat::Result<impl View> {
    Ok(view! {
        card(
            attrs: attributes! { class="h-full" },
            card_header(
                card_title(
                    <span class="flex items-center gap-2">
                        icon(
                            data: icon_data,
                            attrs: attributes! { class="text-header" }
                        )
                        (title)
                    </span>
                )
            )
            card_content(card_description((description)))
            card_footer(
                <a
                    class=(button_variants(ButtonVariant::Outline, ButtonSize::Sm))
                    href=(url)
                >
                    (link_label)
                </a>
            )
        )
    })
}
