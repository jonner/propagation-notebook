use libpropagation::{
    region::{Region, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use topcoat::{
    context::Cx,
    router::{page, path_param, query_params},
    view::view,
};
use tracing::trace;

use crate::{
    components::pagination_control,
    leaflet::Map,
    util::{PageQueryParams, PageState, Path, RegionId, TaxonId, db},
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

#[page("/regions/{region_id}/taxa")]
pub async fn taxa_list(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let region_id = path_param::<RegionId>(cx)?;
    let params = query_params::<PageQueryParams>(cx)?;
    let filter = Taxon::filter(
        Taxon::fields()
            .regional_statuses()
            .any(RegionalTaxonStatus::fields().region_id().eq(region_id)),
    );
    let total = filter.clone().count().exec(&mut db).await? as usize;
    let page_state = PageState::new(params.offset, total);
    let taxa = filter
        .include(Taxon::fields().regional_statuses())
        .order_by(Taxon::fields().sequence().asc())
        .limit(page_state.per_page)
        .offset(page_state.offset)
        .exec(&mut db)
        .await?;
    let region = Region::get_by_id(&mut db, region_id).await?;
    view! {
        <h1>(&region.name)</h1>
        <ul>
            for taxon in taxa {
                for rts in taxon.regional_statuses.get() {
                    if rts.region_id == region.id {
                        <li>
                            <span class="latin">
                                <a href=(rts.path())>(&taxon.complete_name)</a>
                            </span>
                        </li>
                    }
                }
            }
        </ul>
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
        <dd>(rts.origin.map(|v| v.to_string()).unwrap_or_default())</dd>
        <dt>"C-value"</dt>
        <dd>(rts.c_value.map(|v| v.to_string()).unwrap_or_default())</dd>
        <dt>"Conservation Status"</dt>
        <dd>
            (rts
                .conservation_status
                .map(|v| v.to_string())
                .unwrap_or_default())
        </dd>
        <dt>"Wetland Indicator"</dt>
        <dd>
            (rts
                .wetland_indicator
                .map(|v| v.to_string())
                .unwrap_or_default())
        </dd>
        <dt>"Harvest Window"</dt>
        <dd>(rts.harvest_window.to_string())</dd>
    }
}
