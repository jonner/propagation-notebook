use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use libpropagation::{
    region::{Region, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use tracing::trace;

use crate::{
    AppState,
    error::Error,
    templates,
    util::{PER_PAGE, PageQueryParams, PageState},
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_region_list))
        .route("/{id}", get(get_region_details))
        .route("/{id}/taxa", get(get_region_taxa_list))
        .route("/{region_id}/taxa/{taxon_id}", get(get_region_taxon_status))
}

pub async fn get_region_list(State(s): State<Arc<AppState>>) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let regions = Region::all().exec(&mut db).await?;
    trace!(?regions);
    Ok(templates::pages::region::root(&regions))
}

pub async fn get_region_details(
    State(s): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await?;
    trace!(?region);
    Ok(templates::pages::region::details(&region))
}

pub async fn get_region_taxa_list(
    State(s): State<Arc<AppState>>,
    Path(region_id): Path<u64>,
    Query(params): Query<PageQueryParams>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let filter = Taxon::filter(
        Taxon::fields()
            .regional_statuses()
            .any(RegionalTaxonStatus::fields().region_id().eq(region_id)),
    );
    let total = filter.clone().count().exec(&mut db).await? as usize;
    let page_state = PageState {
        per_page: PER_PAGE,
        offset: params.offset.unwrap_or_default(),
        total,
    };
    let taxa = filter
        .include(Taxon::fields().regional_statuses())
        .order_by(Taxon::fields().sequence().asc())
        .limit(page_state.per_page)
        .offset(page_state.offset)
        .exec(&mut db)
        .await?;
    let region = Region::get_by_id(&mut db, region_id).await?;
    Ok(templates::pages::region::taxa_list(
        &region,
        &taxa,
        &page_state,
        &params,
    ))
}

pub async fn get_region_taxon_status(
    State(s): State<Arc<AppState>>,
    Path((region_id, taxon_id)): Path<(u64, u64)>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let rt = RegionalTaxonStatus::filter_by_taxon_id_and_region_id(taxon_id, region_id)
        .include(RegionalTaxonStatus::fields().region())
        .include(RegionalTaxonStatus::fields().taxon().vernaculars())
        .one()
        .exec(&mut db)
        .await?;
    Ok(templates::pages::region::taxon_details(&rt))
}
