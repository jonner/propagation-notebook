use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use libpropagation::{
    collecting::CleaningProcedure,
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{
    AppState,
    error::Error,
    templates,
    util::{ModifyOffset, PageState},
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_taxa_list))
        .route("/{id}", get(get_taxon_details))
        .route(
            "/{taxon_id}/propagation/{propagation_id}",
            get(get_taxon_propagation),
        )
        .route(
            "/{taxon_id}/cleaning/{cleaning_id}",
            get(get_taxon_cleaning),
        )
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaxaListParams {
    offset: Option<usize>,
    #[serde(rename = "q")]
    search_term: Option<String>,
}

impl ModifyOffset for TaxaListParams {
    fn modify_offset(&mut self, new_offset: usize) {
        self.offset = Some(new_offset);
    }
}

pub async fn get_taxa_list(
    Query(params): Query<TaxaListParams>,
    State(s): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let query = match params.search_term.as_ref() {
        Some(q) => Taxon::filter(Taxon::search_filter(q)),
        None => Taxon::all(),
    }
    .order_by((
        Taxon::fields().sequence().asc(),
        Taxon::fields().complete_name().asc(),
    ));
    let total = query.clone().count().exec(&mut db).await? as usize;
    let page_state = PageState::new(params.offset, total);
    let taxa = query
        .limit(page_state.per_page)
        .offset(page_state.offset)
        .exec(&mut db)
        .await?;
    trace!(?taxa);
    Ok(templates::pages::taxonomy::root(
        &taxa,
        &page_state,
        &params,
    ))
}

pub async fn get_taxon_details(
    State(s): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let taxon = Taxon::filter_by_id(id)
        .include(Taxon::fields().vernaculars())
        .include(Taxon::fields().parent())
        .include(Taxon::fields().synonyms())
        .include(Taxon::fields().children())
        .include(Taxon::fields().collecting_data())
        .include(Taxon::fields().cleaning_procedures())
        .include(Taxon::fields().propagation_procedures().propagation())
        .include(Taxon::fields().regional_statuses().region())
        .include(Taxon::fields().notes())
        .one()
        .exec(&mut db)
        .await?;
    trace!(?taxon);
    Ok(templates::pages::taxonomy::details(&taxon))
}

pub async fn get_taxon_propagation(
    State(s): State<Arc<AppState>>,
    Path((taxon_id, propagation_id)): Path<(u64, u64)>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let tp =
        TaxonPropagationProcedure::filter_by_taxon_id_and_propagation_id(taxon_id, propagation_id)
            .include(TaxonPropagationProcedure::fields().propagation())
            .include(TaxonPropagationProcedure::fields().taxon())
            .include(
                TaxonPropagationProcedure::fields()
                    .citation_links()
                    .citation(),
            )
            .one()
            .exec(&mut db)
            .await?;
    trace!(?tp);
    Ok(templates::pages::taxonomy::propagation_details(&tp))
}

pub async fn get_taxon_cleaning(
    State(s): State<Arc<AppState>>,
    Path((taxon_id, cleaning_id)): Path<(u64, u64)>,
) -> Result<impl IntoResponse, Error> {
    let mut db = s.db.clone();
    let proc = CleaningProcedure::filter(
        CleaningProcedure::fields()
            .taxon_id()
            .eq(taxon_id)
            .and(CleaningProcedure::fields().id().eq(cleaning_id)),
    )
    .include(CleaningProcedure::fields().taxon())
    .include(CleaningProcedure::fields().citation_links().citation())
    .one()
    .exec(&mut db)
    .await?;
    trace!(?proc);
    Ok(templates::pages::taxonomy::cleaning_details(&proc))
}
