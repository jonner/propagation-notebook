use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use libpropagation::{
    region::{Region, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use tracing::trace;

use crate::{AppState, error::Error, templates};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handle_root))
        .route("/{id}", get(handle_region_details))
        .route("/{id}/taxa", get(handle_region_taxa_list))
        .route("/{region_id}/taxa/{taxon_id}", get(handle_region_taxon))
}

pub async fn handle_root(State(s): State<Arc<AppState>>) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let regions = Region::all().exec(&mut db).await?;
    trace!(?regions);
    Ok(templates::pages::region::root(&regions))
}

pub async fn handle_region_details(
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

pub async fn handle_region_taxa_list(
    State(s): State<Arc<AppState>>,
    Path(region_id): Path<u64>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let taxa = Taxon::filter(
        Taxon::fields()
            .regional_statuses()
            .any(RegionalTaxonStatus::fields().region_id().eq(region_id)),
    )
    .include(Taxon::fields().regional_statuses())
    .order_by(Taxon::fields().sequence().asc())
    .exec(&mut db)
    .await?;
    let region = Region::get_by_id(&mut db, region_id).await?;
    Ok(templates::pages::region::taxa_list(&region, &taxa))
}

pub async fn handle_region_taxon(
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
