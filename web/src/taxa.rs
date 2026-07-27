use libpropagation::{
    collecting::CleaningProcedure,
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use topcoat::{
    context::Cx,
    router::{page, path_param, query_params},
    view::view,
};
use tracing::trace;

use crate::{
    components::citation_list,
    util::{
        CleaningId, ModifyOffset, PageState, Path, PropagationId, TaxonId, db, pagination_control,
    },
};

#[derive(Debug, Clone, serde::Serialize)]
#[query_params(error = bad_request)]
pub struct TaxaListParams {
    pub offset: Option<usize>,
    pub q: Option<String>,
}

impl ModifyOffset for TaxaListParams {
    fn modify_offset(&mut self, new_offset: usize) {
        self.offset = Some(new_offset);
    }
}

#[page("/taxa")]
pub(crate) async fn list(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let params = query_params::<TaxaListParams>(cx)?;
    let query = match params.q.as_ref() {
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
    view! {
        <h1>"Taxon List"</h1>
        <form method="get">
            <input
                type="text"
                name="q"
                placeholder="Search for a taxon"
                value=(params.q.as_deref().unwrap_or_default())
            >
            <button type="submit">"Search"</button>
        </form>
        <ul>
            for taxon in taxa.iter() {
                <li><a href=(taxon.path())>(&taxon.complete_name)</a></li>
            }
        </ul>
        pagination_control(state: &page_state, params: params)
    }
}

#[page("/taxa/{taxon_id}")]
pub async fn details(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let id = path_param::<TaxonId>(cx)?;
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

    view! {
        <h1>(&taxon.complete_name)</h1>
        <dt>"ID"</dt>
        <dd>(taxon.id)</dd>
        <dt>"Rank"</dt>
        <dd>(taxon.rank.to_string())</dd>

        <dt>"Common Name(s)"</dt>
        <dd>
            <ul>
                for cn in taxon.vernaculars.get() {
                    <li>(&cn.name)</li>
                }
            </ul>
        </dd>

        <dt>"Parent"</dt>
        <dd>
            match taxon.parent.get() {
                Some(p) => <a href=(p.path())>(&p.complete_name)</a>,
                None => "",
            }
        </dd>

        <dt>"Child taxa"</dt>
        <dd>
            <ul>
                for child in taxon.children.get() {
                    <li><a href=(child.path())>(&child.complete_name)</a></li>
                }
            </ul>
        </dd>

        <dt>"Synonyms"</dt>
        <dd>
            <ul>
                for syn in taxon.synonyms.get() {
                    <li>(&syn.complete_name)</li>
                }
            </ul>
        </dd>

        <dt>"ITIS taxon ID"</dt>
        <dd>(taxon.itis_id)</dd>

        <dt>"iNaturalist taxon ID"</dt>
        <dd>
            (taxon
                .inaturalist_id
                .map(|v| v.to_string())
                .unwrap_or_default())
        </dd>
        if let Some(collecting_data) = &taxon.collecting_data.get() {
            <dt>"Ripening"</dt>
            <dd>
                (collecting_data
                    .ripening_indicators
                    .as_deref()
                    .unwrap_or_default())
            </dd>
            <dt>"Harvesting Notes"</dt>
            <dd>
                (collecting_data
                    .harvesting_notes
                    .as_deref()
                    .unwrap_or_default())
            </dd>
            <dt>"Storage Conditions"</dt>
            <dd>(collecting_data.storage.as_deref().unwrap_or_default())</dd>

            <dt>"Storage Life"</dt>
            <dd>
                (collecting_data
                    .storage_life
                    .as_deref()
                    .unwrap_or_default())
            </dd>
        }
        <dt>"Seed Cleaning"</dt>
        <dd>
            <ul>
                for procedure in taxon.cleaning_procedures.get() {
                    <li><a href=(procedure.path())>(&procedure.name)</a></li>
                }
            </ul>
        </dd>
        <dt>"Propagation Procedures"</dt>
        <dd>
            <ul>
                for tp in taxon.propagation_procedures.get() {
                    <li><a href=(tp.path())>(&tp.propagation.get().name)</a></li>
                }
            </ul>
        </dd>
        <dt>"Regions"</dt>
        <dd>
            if !taxon.regional_statuses.get().is_empty() {
                <table>
                    <tr>
                        <th>"ID"</th>
                        <th>"Name"</th>
                        <th>"Origin"</th>
                        <th>"Harvest Window"</th>
                    </tr>
                    for rs in taxon.regional_statuses.get() {
                        <tr>
                            <td>(rs.region.get().id)</td>
                            <td><a href=(rs.path())>(&rs.region.get().name)</a></td>
                            <td>
                                (rs
                                    .origin
                                    .map(|v| v.to_string())
                                    .unwrap_or_default())
                            </td>
                            <td>(rs.harvest_window.to_string())</td>
                        </tr>
                    }
                </table>
            }
        </dd>
        <dt>"Notes"</dt>
        <dd>
            if !taxon.notes.get().is_empty() {
                <table>
                    <tr>
                        <th>"ID"</th>
                        <th>"Name"</th>
                    </tr>
                    for note in taxon.notes.get() {
                        <tr>
                            <td>(note.id)</td>
                            <td>(&note.text)</td>
                        </tr>
                    }
                </table>
            }
        </dd>
    }
}

#[page("/taxa/{taxon_id}/propagation/{propagation_id}")]
pub async fn propagation_details(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let taxon_id = path_param::<TaxonId>(cx)?;
    let propagation_id = path_param::<PropagationId>(cx)?;
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
    let proc = tp.propagation.get();
    let taxon = tp.taxon.get();
    let title = format!("{} for {}", proc.name, taxon.complete_name);

    view! {
        <h1>(title)</h1>
        <dt>"Procedure"</dt>
        <dd><a href=(tp.propagation.get().path())>(&tp.propagation.get().name)</a></dd>
        <dt>"Taxon"</dt>
        <dd><a href=(taxon.path())>(&taxon.complete_name)</a></dd>
        <dt>"Confidence"</dt>
        <dd>(tp.confidence.map(|v| v.to_string()).unwrap_or_default())</dd>
        <dt>"Taxon-specific notes"</dt>
        <dd>(tp.notes.as_deref().unwrap_or_default())</dd>
        <dt>"Citations"</dt>
        <dd>
            citation_list(
                citations: tp
                    .citation_links
                    .get()
                    .iter()
                    .map(|cl| cl.citation.get())
                    .collect()
            )
        </dd>
    }
}

#[page("/taxa/{taxon_id}/cleaning/{cleaning_id}")]
pub async fn cleaning_details(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let taxon_id = path_param::<TaxonId>(cx)?;
    let cleaning_id = path_param::<CleaningId>(cx)?;
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
    let taxon = proc.taxon.get();
    let title = format!("{} for {}", proc.name, taxon.complete_name);
    view! {
        <h1>(title)</h1>
        <dt>"Taxon"</dt>
        <dd><a href=(taxon.path())>(&taxon.complete_name)</a></dd>
        <dt>"Instructions"</dt>
        <dd>(proc.instructions)</dd>
        <dt>"Additional Notes"</dt>
        <dd>(proc.notes.as_deref().unwrap_or_default())</dd>
        <dt>"Citations"</dt>
        <dd>
            citation_list(
                citations: proc
                    .citation_links
                    .get()
                    .iter()
                    .map(|cl| cl.citation.get())
                    .collect()
            )
        </dd>
    }
}
