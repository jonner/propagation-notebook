use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use libpropagation::propagation::PropagationProcedure;
use tracing::trace;

use crate::{AppState, error::Error, templates};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_propagation_list))
        .route("/{id}", get(get_propagation_details))
}

pub async fn get_propagation_list(
    State(s): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let procedures = PropagationProcedure::all().exec(&mut db).await?;
    trace!(?procedures);
    Ok(templates::pages::propagation::propagation_list(&procedures))
}

pub async fn get_propagation_details(
    State(s): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let proc = PropagationProcedure::filter_by_id(id)
        .include(PropagationProcedure::fields().taxa().taxon())
        .include(PropagationProcedure::fields().citations())
        .one()
        .exec(&mut db)
        .await?;
    trace!(?proc);
    Ok(templates::pages::propagation::propagation_details(&proc))
}
