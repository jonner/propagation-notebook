use libpropagation::region::{OriginFilter, Region, RegionCategory, RegionalTaxonStatus};
use serde::Serialize;
use topcoat::{
    context::Cx,
    router::{page, path_param, query_params},
    view::{attributes, view},
};
use tracing::trace;

use crate::{
    components::{
        conservation_status_badge, harvest_timeline, leaflet_map, origin_badge, pagination_control,
        regional_taxa_table,
    },
    util::{ModifyOffset, PER_PAGE, PageState, Path, RegionId, TaxonId, db},
};

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
                            <li><a href=(region.path())>(region.name)</a></li>
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
    let (total_ending, ending) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(libpropagation::region::HarvestFilter::EndingSoon(10)),
            None,
            Some(N),
        )
        .await?;
    let (total_starting, starting_soon) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(libpropagation::region::HarvestFilter::StartingSoon(10)),
            None,
            Some(N),
        )
        .await?;
    view! {
        <h1>(&region.name)</h1>
        <div class="flex flex-col gap-6">
            <section>
                <p>(region.notes.as_deref().unwrap_or_default())</p>
                <nav>
                    <ul>
                        <li>
                            <a href=(format!("/regions/{id}/details"))>
                                "Region details"
                            </a>
                        </li>
                        <li>
                            <a href=(format!("/regions/{id}/taxa"))>
                                "Full taxon list"
                            </a>
                            " ("
                            (region.taxon_statuses.get().len())
                            " taxa)"
                        </li>
                    </ul>
                </nav>
            </section>
            <section>
                <h2>"Search"</h2>
                <form
                    method="get"
                    action=(format!("/regions/{id}/taxa"))
                    class="flex w-full md:w-xl"
                >
                    <input
                        type="text"
                        name="q"
                        placeholder="Search for a taxon"
                        class="me-2 flex-grow"
                    >
                    <button type="submit">"Search"</button>
                </form>
            </section>
            <section>
                <h2>"Last chance to harvest"</h2>
                regional_taxa_table(
                    taxa: &ending,
                    if total_ending > N.try_into().unwrap_or_default() {
                        <div><a href=(format!("/regions/{id}/ending"))>"Full list"</a></div>
                    }
                )
            </section>
            <section>
                <h2>"Beginning to bear fruit"</h2>
                regional_taxa_table(
                    taxa: &starting_soon,
                    if total_starting > N.try_into().unwrap_or_default() {
                        <div>
                            <a href=(format!("/regions/{id}/starting"))>"Full list"</a>
                        </div>
                    }
                )
            </section>
        </div>
    }
}

#[page("/regions/{region_id}/starting")]
pub(crate) async fn harvest_starting(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let mut db = db(cx);
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await?;
    // FIXME: pagination
    let (_total, starting_soon) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(libpropagation::region::HarvestFilter::StartingSoon(10)),
            None,
            None,
        )
        .await?;
    view! {
        <h1>(&region.name)</h1>
        <h2>"Taxa beginning to bear fruit"</h2>
        regional_taxa_table(taxa: &starting_soon)
    }
}

#[page("/regions/{region_id}/ending")]
pub(crate) async fn harvest_ending(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let mut db = db(cx);
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await?;
    // FIXME: pagination
    let (_total, ending) = region
        .get_taxa(
            &mut db,
            None,
            Some(OriginFilter::NativeOnly),
            Some(libpropagation::region::HarvestFilter::EndingSoon(10)),
            None,
            Some(10),
        )
        .await?;
    view! {
        <h1>(&region.name)</h1>
        <h2>"Last chance to harvest"</h2>
        regional_taxa_table(taxa: &ending)
    }
}

#[page("/regions/{region_id}/details")]
pub(crate) async fn details(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let mut db = db(cx);
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await?;
    trace!(?region);
    view! {
        <div class="flex flex-col gap-6">
            <hgroup>
                <h1>(&region.name)</h1>
                <p>(region.notes.as_deref().unwrap_or_default())</p>
            </hgroup>
            <section>
                <h2>"Category"</h2>
                <div>(region.category.to_string())</div>
            </section>
            <section>
                <h2>"Taxa"</h2>
                <div>
                    <a href=(format!("./{}/taxa", region.id))>
                        (region.taxon_statuses.get().len())
                    </a>
                </div>
            </section>
            if let Some(value) = region.geometry.as_ref() {
                <section>
                    <h2>"Geometry"</h2>
                    <div>
                        leaflet_map(
                            geometry: value,
                            attrs: attributes!(class="w-full aspect-square max-h-[50dvh]")
                        )
                    </div>
                </section>
            }
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
                Some(libpropagation::region::HarvestFilter::ReadyNow)
            } else {
                None
            },
            params.offset,
            Some(PER_PAGE),
        )
        .await?;
    let page_state = PageState::new(params.offset, PER_PAGE, total.try_into().unwrap());
    view! {
        <h1>(&region.name)</h1>
        <section>
            <h2>"Search"</h2>
            <form method="get" class="flex w-full md:w-xl">
                <input
                    type="text"
                    name="q"
                    value=(params.q.as_ref())
                    placeholder="Search for a taxon"
                    class="me-2 flex-grow"
                >
                <button type="submit">"Search"</button>
            </form>
        </section>
        <section>
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            regional_taxa_table(taxa: &taxa)
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
        </section>
    }
}

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
            <span class="latin"><a href=(taxon.path())>(&taxon.complete_name)</a></span>
        </dd>
        <dt>"Region"</dt>
        <dd><a href=(region.path())>(&region.name)</a></dd>
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
            <div class="flex items-center gap-x-6">
                harvest_timeline(
                    window: &rts.harvest_window,
                    attrs: attributes! { class="w-md" }
                )
                (rts.harvest_window.to_string())
            </div>
        </dd>
    }
}
