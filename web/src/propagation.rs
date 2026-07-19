use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use libpropagation::propagation::PropagationProcedure;
use tracing::trace;

use crate::{AppState, templates};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handle_root))
        .route("/{id}", get(handle_propagation_details))
}

pub async fn handle_root(State(s): State<Arc<AppState>>) -> impl IntoResponse {
    let mut db = s.db.clone();
    let procedures = PropagationProcedure::all().exec(&mut db).await.unwrap();
    trace!(?procedures);
    templates::pages::propagation::root(&procedures)
}

pub async fn handle_propagation_details(
    State(s): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let mut db = s.db.clone();
    let proc = PropagationProcedure::filter_by_id(id)
        .include(PropagationProcedure::fields().taxa().taxon())
        .include(PropagationProcedure::fields().citations())
        .one()
        .exec(&mut db)
        .await
        .unwrap();
    trace!(?proc);
    templates::pages::propagation::details(&proc)
}
