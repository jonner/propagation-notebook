use libpropagation::{
    region::{HarvestFilter, OriginFilter, Region, RegionCategory, RegionalTaxonStatus},
    taxonomy::{Rank, Taxon, TaxonHierarchy},
};
use serde::Serialize;
use topcoat::{
    context::Cx,
    router::{error::RouterErrorExt, href, page, path_param, query_params},
    view::{attributes, view},
};
use tracing::trace;

use crate::{
    components::{
        badge::{conservation_status_badge, origin_badge},
        breadcrumb::*,
        button::button,
        card::*,
        harvest::{harvest_timeline, regional_taxa_table},
        input::input,
        leaflet_map,
        pagination::{pagination_control, week_navigator},
    },
    taxa::{self, TaxaListParams, taxonomy},
    util::{ModifyOffset, PER_PAGE, PageState, db},
};

path_param!(pub region_id: u64, error = bad_request);

const SEED_DISCLAIMER: &str = "Seed dates are based on data retrieved from iNaturalist.org. If a species does not have any information about its seed-bearing window shown here, it is likely that either iNaturalist doesn't have enough observations of the species within the given area, or too few observations are annotated to indicate the presence of seeds. We periodically import new data from iNaturalist, so any new or newly-annotated observations should improve these dates over time.";

#[page("/regions")]
pub async fn list(cx: &Cx) -> topcoat::Result {
    let db = db(cx);
    let mut db = db;
    let mut region_types = Vec::default();
    Region::filter(Region::fields().category().eq(RegionCategory::Nation))
        .order_by(Region::fields().name().asc())
        .exec(&mut db)
        .await
        .map(|r| {
            if !r.is_empty() {
                region_types.push(("Countries", r))
            }
        })?;
    Region::filter(Region::fields().category().eq(RegionCategory::Province))
        .order_by(Region::fields().name().asc())
        .exec(&mut db)
        .await
        .map(|r| {
            if !r.is_empty() {
                region_types.push(("States and Provinces", r))
            }
        })?;
    Region::filter(Region::fields().category().eq(RegionCategory::County))
        .order_by(Region::fields().name().asc())
        .exec(&mut db)
        .await
        .map(|r| {
            if !r.is_empty() {
                region_types.push(("Counties or Districts", r))
            }
        })?;
    Region::filter(Region::fields().category().eq(RegionCategory::Municipality))
        .order_by(Region::fields().name().asc())
        .exec(&mut db)
        .await
        .map(|r| {
            if !r.is_empty() {
                region_types.push(("Cities or Municipalities", r))
            }
        })?;
    Region::filter(Region::fields().category().eq(RegionCategory::Other))
        .order_by(Region::fields().name().asc())
        .exec(&mut db)
        .await
        .map(|r| {
            if !r.is_empty() {
                region_types.push(("Other", r))
            }
        })?;
    trace!(?region_types);
    view! {
        <h1>"Regions"</h1>
        <div class="flex flex-col gap-4">
            for t in region_types {
                let regions = t.1;
                <section>
                    <h2>(t.0)</h2>
                    <ul class="contents">
                        for region in regions {
                            <li>
                                <a href=(href!(overview, RegionId(region.id)))>
                                    (region.name)
                                </a>
                            </li>
                        }
                    </ul>
                </section>
            }
        </div>
    }
}

#[page("/regions/{region_id}")]
pub(crate) async fn overview(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let mut db = db(cx);
    const N: usize = 10;
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await
        .ok_or_not_found()?;

    let nspecies = Taxon::filter(
        Taxon::fields().rank().eq(Rank::Species).and(
            Taxon::fields().descendant_links().any(
                TaxonHierarchy::fields()
                    .descendant()
                    .regional_statuses()
                    .any(RegionalTaxonStatus::fields().region_id().eq(id)),
            ),
        ),
    )
    .count()
    .exec(&mut db)
    .await?;
    let (n_fruiting, _) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::ReadyNow),
            None,
            Some(N),
        )
        .await?;
    let (n_ending, _) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::EndingSoon {
                context: 10,
                doy: None,
            }),
            None,
            Some(N),
        )
        .await?;
    let (n_starting, _) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::StartingSoon {
                context: 10,
                doy: None,
            }),
            None,
            Some(N),
        )
        .await?;

    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(list)) },
                                "Region"
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page("Overview"))
                    )
                )
                <h1>(&region.name)</h1>
                <p>(region.notes.as_deref().unwrap_or_default())</p>
            </hgroup>
            <section>
                <form
                    method="get"
                    action=(href!(fruiting_all, RegionId(region.id)))
                    class="flex w-full"
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
            <div class="flex flex-col md:flex-row gap-6">
                if let Some(value) = region.geometry.as_ref() {
                    <section class="grow order-last md:min-w-2/3 md:order-first">
                        <div>
                            leaflet_map(
                                geometry: value,
                                attrs: attributes!(class="w-full aspect-square")
                            )
                        </div>
                    </section>
                }
                <section
                    class="flex flex-col shrink items-stretch gap-6 justify-between"
                >
                    card(
                        attrs: attributes! { class="grow" },
                        card_header(card_title("Region Stats"))
                        card_content(
                            attrs: attributes! { class="grow" },
                            <div class="flex flex-col gap-3">
                                <div>
                                    <h4>"Species"</h4>
                                    (nspecies)
                                </div>
                                <div>
                                    <h4>"Total Taxa"</h4>
                                    (region.taxon_statuses.get().len())
                                </div>
                            </div>
                        )
                        card_footer(
                            <a
                                href=(href!(taxonomy).query(
                                    TaxaListParams {
                                        parent: None,
                                        fmt: None,
                                        offset: None,
                                        region: Some(region.id),
                                    },
                                ))
                            >
                                "Browse all taxa"
                            </a>
                        )
                    )
                    card(
                        attrs: attributes! { class="grow" },
                        card_header(
                            card_title("Seed-bearing status")
                            card_description(
                                attrs: attributes! { class="whitespace-normal" },
                                "Using data from iNaturalist, we can estimate when a taxon will be producing seed within this region."
                            )
                        )
                        card_content(
                            attrs: attributes! { class="grow" },
                            <div class="flex flex-col gap-3">
                                <div>
                                    <h4>"Currently Fruiting"</h4>
                                    <a href=(href!(fruiting_now, RegionId(region.id)))>
                                        (n_fruiting)
                                    </a>
                                </div>
                                <div>
                                    <h4>"Winding down..."</h4>
                                    <a href=(href!(fruiting_done, RegionId(region.id)))>
                                        (n_ending)
                                    </a>
                                </div>
                                <div>
                                    <h4>"Ramping up..."</h4>
                                    <a href=(href!(fruiting_soon, RegionId(region.id)))>
                                        (n_starting)
                                    </a>
                                </div>
                            </div>
                        )
                        card_footer(
                            <a href=(href!(fruiting_all, RegionId(region.id)))>
                                "All seed-bearing statuses"
                            </a>
                        )
                    )
                </section>
            </div>
        </div>
    }
}

#[derive(Debug, Clone, Serialize)]
#[query_params(error = bad_request)]
struct HarvestParams {
    date: Option<jiff::civil::Date>,
    offset: Option<usize>,
}

impl ModifyOffset for HarvestParams {
    fn modify_offset(&mut self, new_offset: usize) {
        self.offset = Some(new_offset);
    }
}

#[page("/regions/{region_id}/fruiting/soon")]
pub(crate) async fn fruiting_soon(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let mut db = db(cx);
    let params = query_params::<HarvestParams>(cx)?;
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await
        .ok_or_not_found()?;
    let date = params.date.unwrap_or_else(|| jiff::Zoned::now().date());
    let doy = date.day_of_year();
    let (total, starting_soon) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::StartingSoon {
                context: 10,
                doy: Some(doy),
            }),
            params.offset,
            Some(PER_PAGE),
        )
        .await?;
    let page_state = PageState::new(params.offset, PER_PAGE, total.try_into().unwrap());

    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(list)) },
                                "Region"
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(overview, RegionId(region.id))) },
                                (region.name)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(fruiting_all, RegionId(region.id))) },
                                "Seed-bearing"
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page("Soon"))
                    )
                )
                <h1>"Taxa beginning to bear fruit"</h1>
                <p>
                    "The following species should be starting to bear fruit on the given date."
                </p>
                week_navigator(date: date)
            </hgroup>
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            regional_taxa_table(taxa: &starting_soon, current_doy: Some(doy))
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            <div class="disclaimer">(SEED_DISCLAIMER)</div>
        </div>
    }
}

#[page("/regions/{region_id}/fruiting/done")]
pub(crate) async fn fruiting_done(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let params = query_params::<HarvestParams>(cx)?;
    let mut db = db(cx);
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await
        .ok_or_not_found()?;
    let date = params.date.unwrap_or_else(|| jiff::Zoned::now().date());
    let doy = date.day_of_year();
    let (total, ending) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::EndingSoon {
                context: 10,
                doy: Some(doy),
            }),
            params.offset,
            Some(PER_PAGE),
        )
        .await?;
    let page_state = PageState::new(params.offset, PER_PAGE, total.try_into().unwrap());

    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(list)) },
                                "Region"
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(overview, RegionId(region.id))) },
                                (region.name)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(fruiting_all, RegionId(region.id))) },
                                "Seed-bearing"
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page("Ending"))
                    )
                )
                <h1>"Last chance to harvest"</h1>
                <p>
                    ("These species are nearly done bearing fruit soon on the given date.")
                </p>
                week_navigator(date: date)
            </hgroup>
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            regional_taxa_table(taxa: &ending, current_doy: Some(doy))
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            <div class="disclaimer">(SEED_DISCLAIMER)</div>
        </div>
    }
}

#[page("/regions/{region_id}/fruiting/now")]
pub(crate) async fn fruiting_now(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let params = query_params::<HarvestParams>(cx)?;
    let mut db = db(cx);
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await
        .ok_or_not_found()?;
    let date = params.date.unwrap_or_else(|| jiff::Zoned::now().date());
    let doy = date.day_of_year();
    let (total, ending) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::ReadyOnDoy(date.day_of_year())),
            params.offset,
            Some(PER_PAGE),
        )
        .await?;
    let page_state = PageState::new(params.offset, PER_PAGE, total.try_into().unwrap());

    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(list)) },
                                "Region"
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(overview, RegionId(region.id))) },
                                (region.name)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(fruiting_all, RegionId(region.id))) },
                                "Seed-bearing"
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page("Now"))
                    )
                )
                <h1>"Currently fruiting"</h1>
                <p>
                    ("These species are likely to be bearing seeds on the given date. Note that this does not necessarily mean that the seed is 'ripe' or viable yet.")
                </p>
                week_navigator(
                    date: date,
                    attrs: attributes! { class="justify-center md:justify-normal" }
                )
            </hgroup>
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            regional_taxa_table(taxa: &ending, current_doy: Some(doy))
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            <div class="disclaimer">(SEED_DISCLAIMER)</div>
        </div>
    }
}

#[derive(Debug, Clone, Serialize)]
#[query_params(error = bad_request)]
pub struct RegionalTaxaListParams {
    pub offset: Option<usize>,
    pub ready: Option<bool>,
    pub native: Option<bool>,
    pub q: Option<String>,
}

impl ModifyOffset for RegionalTaxaListParams {
    fn modify_offset(&mut self, new_offset: usize) {
        self.offset = Some(new_offset)
    }
}

#[page("/regions/{region_id}/fruiting")]
pub async fn fruiting_all(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let region_id = path_param::<RegionId>(cx)?;
    let params = query_params::<RegionalTaxaListParams>(cx)?;
    let region = Region::get_by_id(&mut db, region_id)
        .await
        .ok_or_not_found()?;
    let (total, taxa) = region
        .get_taxa(
            &mut db,
            params.q.as_ref(),
            if Some(true) == params.native {
                Some(OriginFilter::NativeOnly)
            } else {
                None
            },
            if Some(true) == params.ready {
                Some(HarvestFilter::ReadyNow)
            } else {
                None
            },
            params.offset,
            Some(PER_PAGE),
        )
        .await?;
    let page_state = PageState::new(params.offset, PER_PAGE, total.try_into().unwrap());

    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(list)) },
                                "Region"
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(overview, RegionId(region.id))) },
                                (&region.name)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page("Seed-bearing"))
                    )
                )
                <h1>(format!("Seed-bearing status for plants in {}", &region.name))</h1>
                <p>
                    "Seed-bearing timeline for all plants within the region. The bar graph represents a full year. A green background indicates that seeds may be present at that time of year. The vertical bar represents today."
                </p>
            </hgroup>
            <section>
                <form method="get" class="flex w-full">
                    input(
                        attrs: attributes! {
                            type="text"
                            name="q"
                            value=(params.q.as_ref())
                            placeholder="Search for a taxon"
                            class="me-2 grow"
                        }
                    )
                    button(attrs: attributes! { type="submit" }, "Search")
                </form>
            </section>
            if let Some(q) = params.q.as_ref() && !q.is_empty() {
                <h2>(format!("Results for '{q}'"))</h2>
            }
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            regional_taxa_table(taxa: &taxa)
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            <div class="disclaimer">(SEED_DISCLAIMER)</div>
        </div>
    }
}

path_param!(taxon_id: u64, error = bad_request);

#[page("/regions/{region_id}/taxa/{taxon_id}")]
pub async fn taxon_status(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let region_id = path_param::<RegionId>(cx).ok_or_not_found()?;
    let taxon_id = path_param::<TaxonId>(cx).ok_or_not_found()?;
    let rts = RegionalTaxonStatus::filter_by_taxon_id_and_region_id(taxon_id, region_id)
        .include(RegionalTaxonStatus::fields().region())
        .include(RegionalTaxonStatus::fields().taxon().vernaculars())
        .one()
        .exec(&mut db)
        .await?;
    let region = rts.region.get();
    let taxon = rts.taxon.get();

    view! {
        <h1>
            <span class="latin">(&taxon.complete_name)</span>
            " in "
            <span>(&region.name)</span>
        </h1>
        <dt>"Taxon"</dt>
        <dd>
            <span class="latin">
                <a href=(href!(taxa::details, taxa::TaxonId(taxon.id)))>
                    (&taxon.complete_name)
                </a>
            </span>
        </dd>
        <dt>"Region"</dt>
        <dd><a href=(href!(overview, RegionId(region.id)))>(&region.name)</a></dd>
        <dt>"Origin"</dt>
        <dd>
            if let Some(origin) = rts.origin {
                origin_badge(origin: origin)
            }
        </dd>
        <dt>"C-value"</dt>
        <dd>(rts.c_value.map(|v| v.to_string()).unwrap_or_default())</dd>
        <dt>"Conservation Status"</dt>
        <dd>
            if let Some(status) = rts.conservation_status {
                conservation_status_badge(status: status)
            }
        </dd>
        <dt>"Wetland Indicator"</dt>
        <dd>(rts.wetland_indicator.map(|v| v.to_string()).unwrap_or_default())</dd>
        <dt>"Fruiting window"</dt>
        <dd>
            <div class="flex h-full items-center gap-x-6">
                harvest_timeline(
                    window: &rts.harvest_window,
                    attrs: attributes! { class="w-md" }
                )
                (rts.harvest_window.to_string())
            </div>
        </dd>
    }
}
