use libpropagation::region::{OriginFilter, Region, RegionalTaxonStatus};
use serde::Serialize;
use topcoat::{
    context::Cx,
    router::{page, path_param, query_params},
    view::{attributes, view},
};
use tracing::trace;

use crate::{
    components::{
        conservation_status_badge, harvest_timeline, origin_badge, pagination_control, taxa_table,
    },
    leaflet::Map,
    util::{ModifyOffset, PageState, Path, RegionId, TaxonId, db},
};

#[page("/regions")]
pub async fn list(cx: &Cx) -> topcoat::Result {
    let db = db(cx);
    let mut db = db;
    let regions = Region::all().exec(&mut db).await?;
    trace!(?regions);
    view! {
        <h1>"Regions"</h1>
        <ul>
            for region in regions {
                <li><a href=(region.path())>(region.name)</a></li>
            }
        </ul>
    }
}

#[page("/regions/{region_id}")]
pub(crate) async fn overview(cx: &Cx) -> topcoat::Result {
    let id = path_param::<RegionId>(cx)?;
    let mut db = db(cx);
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await?;
    let ending = region
        .get_taxa(
            &mut db,
            Some(OriginFilter::NativeOnly),
            Some(libpropagation::region::HarvestFilter::EndingSoon(10)),
            None,
            Some(10),
        )
        .await?;
    let starting_soon = region
        .get_taxa(
            &mut db,
            Some(OriginFilter::NativeOnly),
            Some(libpropagation::region::HarvestFilter::StartingSoon(10)),
            None,
            Some(10),
        )
        .await?;
    view! {
        <h1>(&region.name)</h1>
        <div class="flex flex-col gap-6">
            <div>(region.notes.as_deref().unwrap_or_default())</div>
            <nav>
                <ul>
                    <li>
                        <a href=(format!("/regions/{id}/details"))>"Region details"</a>
                    </li>
                    <li>
                        <a href=(format!("/regions/{id}/taxa"))>"Full taxon list"</a>
                    </li>
                </ul>
            </nav>
            <div>
                <h2>"Last chance to harvest"</h2>
                taxa_table(
                    taxa: ending,
                    <div><a href=(format!("/regions/{id}/ending"))>"Full list"</a></div>
                )
            </div>
            <div>
                <h2>"Beginning to bear fruit"</h2>
                taxa_table(
                    taxa: starting_soon,
                    <div>
                        <a href=(format!("/regions/{id}/starting"))>"Full list"</a>
                    </div>
                )
            </div>
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
    let starting_soon = region
        .get_taxa(
            &mut db,
            Some(OriginFilter::NativeOnly),
            Some(libpropagation::region::HarvestFilter::StartingSoon(10)),
            None,
            None,
        )
        .await?;
    view! {
        <h1>(&region.name)</h1>
        <h2>"Taxa coming into harvest season"</h2>
        taxa_table(taxa: starting_soon)
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
    let ending = region
        .get_taxa(
            &mut db,
            Some(OriginFilter::NativeOnly),
            Some(libpropagation::region::HarvestFilter::EndingSoon(10)),
            None,
            Some(10),
        )
        .await?;
    view! {
        <h1>(&region.name)</h1>
        <h2>"Taxa nearing the end of harvest season"</h2>
        taxa_table(taxa: ending)
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
        <h1>(&region.name)</h1>
        <dl>
            <dt>"ID"</dt>
            <dd>(region.id)</dd>
            <dt>"Category"</dt>
            <dd>(region.category.to_string())</dd>
            <dt>"Notes"</dt>
            <dd>(region.notes.as_deref().unwrap_or_default())</dd>
            <dt>"Taxa"</dt>
            <dd>
                <a href=(format!("./{}/taxa", region.id))>
                    (region.taxon_statuses.get().len())
                </a>
            </dd>
            <dt>"Geometry"</dt>
            <dd>
                match region.geometry.as_ref() {
                    Some(value) => (Map {
                        geometry: value,
                        width: None,
                        height: None,
                    }),
                    None => "",
                }
            </dd>
        </dl>
    }
}

#[derive(Debug, Clone, Serialize)]
#[query_params(error = bad_request)]
pub struct RegionalTaxaListParams {
    pub offset: Option<usize>,
    pub ready: Option<bool>,
    pub native: Option<bool>,
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
    let region = Region::filter_by_id(region_id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await?;
    let total = region.taxon_statuses.get().len();
    let page_state = PageState::new(params.offset, total);
    let taxa = region
        .get_taxa(
            &mut db,
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
            Some(page_state.offset),
            Some(page_state.per_page),
        )
        .await?;
    view! {
        <h1>(&region.name)</h1>
        taxa_table(taxa: taxa)
        pagination_control(state: &page_state, params: params)
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
