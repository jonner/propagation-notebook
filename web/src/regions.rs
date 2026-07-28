use libpropagation::{
    region::{Origin, Region, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use serde::Serialize;
use topcoat::{
    context::Cx,
    router::{page, path_param, query_params},
    view::{attributes, view},
};
use tracing::trace;

use crate::{
    components::pagination_control,
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

#[derive(Debug, Clone, Serialize)]
#[query_params(error = bad_request)]
pub struct RegionalTaxaListParams {
    pub offset: Option<usize>,
    pub ready: Option<bool>,
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
    let mut rts_filter = RegionalTaxonStatus::fields().region_id().eq(region_id);
    if params.ready == Some(true) {
        let day = jiff::Zoned::now().date().day_of_year();
        // include species that start harvesting in the next week
        let start = day;
        // include species that finished harvesting a week ago
        let end = day;
        rts_filter = rts_filter.and(
            RegionalTaxonStatus::fields()
                .harvest_window()
                .start_doy()
                .le(start)
                .and(
                    RegionalTaxonStatus::fields()
                        .harvest_window()
                        .end_doy()
                        .ge(end),
                )
                .or(RegionalTaxonStatus::fields()
                    .harvest_window()
                    .start_doy()
                    .gt(RegionalTaxonStatus::fields().harvest_window().end_doy())
                    .and(
                        RegionalTaxonStatus::fields()
                            .harvest_window()
                            .start_doy()
                            .le(start)
                            .or(RegionalTaxonStatus::fields()
                                .harvest_window()
                                .end_doy()
                                .ge(end)),
                    )),
        );
    }
    let filter = Taxon::filter(Taxon::fields().regional_statuses().any(rts_filter));
    let total = filter.clone().count().exec(&mut db).await? as usize;
    let page_state = PageState::new(params.offset, total);
    let taxa = filter
        .include(
            Taxon::fields()
                .regional_statuses()
                .filter(RegionalTaxonStatus::fields().region_id().eq(region_id)),
        )
        .order_by(Taxon::fields().sequence().asc())
        .limit(page_state.per_page)
        .offset(page_state.offset)
        .exec(&mut db)
        .await?;
    let region = Region::get_by_id(&mut db, region_id).await?;
    view! {
        <h1>(&region.name)</h1>
        <table>
            <tr>
                <th>"Taxon"</th>
                <th>"Origin"</th>
                <th>"Harvest Dates"</th>
            </tr>
            for taxon in taxa {
                if let Some(rts) = taxon.regional_statuses.get().first() {
                    <tr>
                        <td>
                            <span class="latin">
                                <a href=(taxon.path())>(&taxon.complete_name)</a>
                            </span>
                        </td>
                        <td>
                            if let Some(origin) = rts.origin {
                                let attrs = attributes! {
                                    match origin {
                                        Origin::Introduced => class="introduced",
                                        Origin::Native => class="native",
                                        _ => class="",
                                    }
                                };
                                <span (attrs)>(origin.to_string())</span>
                            }
                        </td>
                        <td>(rts.harvest_window.to_string())</td>
                    </tr>
                }
            }
        </table>
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
