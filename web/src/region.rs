use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use libpropagation::region::Region;
use tracing::trace;

use crate::{AppState, error::Error, templates};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handle_root))
        .route("/{id}", get(handle_region_details))
}

pub async fn handle_root(State(s): State<Arc<AppState>>) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let regions = Region::all().exec(&mut db).await?;
    trace!(?regions);
    Ok(templates::pages::regions::root(&regions))
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
    Ok(templates::pages::regions::details(&region))
}
