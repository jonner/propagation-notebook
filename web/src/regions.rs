use libpropagation::region::{
    HarvestFilter, OriginFilter, Region, RegionCategory, RegionalTaxonStatus,
};
use serde::Serialize;
use topcoat::{
    context::Cx,
    router::{href, page, path_param, query_params},
    view::{attributes, view},
};
use tracing::trace;

use crate::{
    components::{
        Breadcrumb,
        badge::{conservation_status_badge, origin_badge},
        breadcrumbs,
        button::button,
        card::{card, card_content, card_footer, card_header, card_title},
        harvest::{harvest_timeline, regional_taxa_table},
        input::input,
        leaflet_map,
        pagination::{pagination_control, week_navigator},
    },
    taxa,
    util::{ModifyOffset, PER_PAGE, PageState, db},
};

path_param!(pub region_id: u64, error = bad_request);

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
    let items = vec![
        Breadcrumb {
            url: Some("/regions".to_string()),
            text: "Regions".to_string(),
        },
        Breadcrumb {
            url: None,
            text: "Overview".to_string(),
        },
    ];
    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumbs(items: items)
                <h1>(&region.name)</h1>
                <p>(region.notes.as_deref().unwrap_or_default())</p>
            </hgroup>
            <section>
                <h2>"Search"</h2>
                <form
                    method="get"
                    action=(href!(taxa_list, RegionId(region.id)))
                    class="flex w-full md:w-2xl"
                >
                    input(
                        attrs: attributes! {
                            type="text"
                            name="q"
                            placeholder="Search for a taxon"
                            class="me-2 flex-grow"
                        }
                    )
                    button(attrs: attributes! { type="submit" }, "Search")
                </form>
            </section>
            <div class="flex flex-col md:flex-row gap-6">
                if let Some(value) = region.geometry.as_ref() {
                    <section class="w-full md:w-2xl order-last md:order-first">
                        <div>
                            leaflet_map(
                                geometry: value,
                                attrs: attributes!(class="w-full aspect-square")
                            )
                        </div>
                    </section>
                }
                <section class="flex flex-col items-stretch gap-6 justify-between">
                    card(
                        attrs: attributes! { class="grow" },
                        card_header(card_title("Total taxa"))
                        card_content(
                            <div>
                                <a
                                    href=(href!(taxa_list, RegionId(region.id)))
                                    class="text-4xl"
                                >
                                    (region.taxon_statuses.get().len())
                                </a>
                            </div>
                        )
                        card_footer()
                    )
                    card(
                        attrs: attributes! { class="grow" },
                        card_header(card_title("Currently Fruiting"))
                        card_content(
                            <div>
                                <a
                                    href=(href!(harvest_fruiting, RegionId(region.id)))
                                    class="text-4xl"
                                >
                                    (n_fruiting)
                                </a>
                            </div>
                        )
                        card_footer()
                    )
                    card(
                        attrs: attributes! { class="grow" },
                        card_header(card_title("Winding Down..."))
                        card_content(
                            <div>
                                <a
                                    href=(href!(harvest_ending, RegionId(region.id)))
                                    class="text-4xl"
                                >
                                    (n_ending)
                                </a>
                            </div>
                        )
                        card_footer()
                    )
                    card(
                        attrs: attributes! { class="grow" },
                        card_header(card_title("Ramping up..."))
                        card_content(
                            <div>
                                <a
                                    href=(href!(harvest_starting, RegionId(region.id)))
                                    class="text-4xl"
                                >
                                    (n_starting)
                                </a>
                            </div>
                        )
                        card_footer()
                    )
                </section>
            </div>
        </div>
    }
}

#[derive(Debug, Serialize)]
#[query_params(error = bad_request)]
struct HarvestParams {
    date: Option<jiff::civil::Date>,
}

#[page("/regions/{region_id}/starting")]
pub(crate) async fn harvest_starting(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let mut db = db(cx);
    let params = query_params::<HarvestParams>(cx)?;
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await?;
    let date = params.date.unwrap_or_else(|| jiff::Zoned::now().date());
    let doy = date.day_of_year();
    // FIXME: pagination
    let (_total, starting_soon) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::StartingSoon {
                context: 10,
                doy: Some(doy),
            }),
            None,
            None,
        )
        .await?;
    let items = vec![
        Breadcrumb {
            url: Some("/regions".to_string()),
            text: "Regions".to_string(),
        },
        Breadcrumb {
            url: Some(href!(overview, RegionId(region.id)).resolve(cx)),
            text: region.name,
        },
        Breadcrumb {
            url: None,
            text: "Starting".to_string(),
        },
    ];
    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumbs(items: items)
                <h1>"Taxa beginning to bear fruit"</h1>
                <p>
                    ("These species will be starting to bear fruit on the given date.")
                </p>
                week_navigator(date: date)
            </hgroup>
            regional_taxa_table(taxa: &starting_soon, current_doy: Some(doy))
        </div>
    }
}

#[page("/regions/{region_id}/ending")]
pub(crate) async fn harvest_ending(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let params = query_params::<HarvestParams>(cx)?;
    let mut db = db(cx);
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await?;
    let date = params.date.unwrap_or_else(|| jiff::Zoned::now().date());
    let doy = date.day_of_year();
    // FIXME: pagination
    let (_total, ending) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::EndingSoon {
                context: 10,
                doy: Some(doy),
            }),
            None,
            None,
        )
        .await?;
    let items = vec![
        Breadcrumb {
            url: Some("/regions".to_string()),
            text: "Regions".to_string(),
        },
        Breadcrumb {
            url: Some(href!(overview, RegionId(region.id)).resolve(cx)),
            text: region.name,
        },
        Breadcrumb {
            url: None,
            text: "Ending".to_string(),
        },
    ];
    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumbs(items: items)
                <h1>"Last chance to harvest"</h1>
                <p>
                    ("These species are nearly done bearing fruit soon on the given date.")
                </p>
                week_navigator(date: date)
            </hgroup>
            regional_taxa_table(taxa: &ending, current_doy: Some(doy))
        </div>
    }
}

#[page("/regions/{region_id}/fruiting")]
pub(crate) async fn harvest_fruiting(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let params = query_params::<HarvestParams>(cx)?;
    let mut db = db(cx);
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await?;
    let date = params.date.unwrap_or_else(|| jiff::Zoned::now().date());
    let doy = date.day_of_year();
    // FIXME: pagination
    let (_total, ending) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(HarvestFilter::ReadyOnDoy(date.day_of_year())),
            None,
            None,
        )
        .await?;
    let items = vec![
        Breadcrumb {
            url: Some("/regions".to_string()),
            text: "Regions".to_string(),
        },
        Breadcrumb {
            url: Some(href!(overview, RegionId(region.id)).resolve(cx)),
            text: region.name,
        },
        Breadcrumb {
            url: None,
            text: "Fruiting".to_string(),
        },
    ];
    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumbs(items: items)
                <h1>"Currently fruiting"</h1>
                <p>("These species are bearing fruit on the given date.")</p>
                week_navigator(
                    date: date,
                    attrs: attributes! { class="justify-center md:justify-normal" }
                )
            </hgroup>
            regional_taxa_table(taxa: &ending, current_doy: Some(doy))
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

#[page("/regions/{region_id}/taxa")]
pub async fn taxa_list(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let region_id = path_param::<RegionId>(cx)?;
    let params = query_params::<RegionalTaxaListParams>(cx)?;
    let region = Region::get_by_id(&mut db, region_id).await?;
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
    let items = vec![
        Breadcrumb {
            url: Some("/regions".to_string()),
            text: "Regions".to_string(),
        },
        Breadcrumb {
            url: Some(href!(overview, RegionId(region.id)).resolve(cx)),
            text: region.name.clone(),
        },
        Breadcrumb {
            url: None,
            text: "Taxa".to_string(),
        },
    ];
    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                breadcrumbs(items: items)
                <h1>(format!("Taxa list for {}", &region.name))</h1>
            </hgroup>
            <section>
                <form method="get" class="flex w-full md:w-xl">
                    input(
                        attrs: attributes! {
                            type="text"
                            name="q"
                            value=(params.q.as_ref())
                            placeholder="Search for a taxon"
                            class="me-2 flex-grow"
                        }
                    )
                    button(attrs: attributes! { type="submit" }, "Search")
                </form>
            </section>
            <section>
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
            </section>
        </div>
    }
}

path_param!(taxon_id: u64, error = bad_request);

#[page("/regions/{region_id}/taxa/{taxon_id}")]
pub async fn taxon_status(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let region_id = path_param::<RegionId>(cx)?;
    let taxon_id = path_param::<TaxonId>(cx)?;
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
        <dd>
            (rts
                .wetland_indicator
                .map(|v| v.to_string())
                .unwrap_or_default())
        </dd>
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
