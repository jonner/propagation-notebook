use libpropagation::{
    propagation::PropagationProcedure,
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use topcoat::{
    context::Cx,
    router::{page, path_param},
    view::view,
};
use tracing::trace;

use crate::{
    components::citation_list,
    util::{Path, PropagationId, db},
};

#[page("/propagation")]
pub async fn get_propagation_list(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let procedures = PropagationProcedure::all().exec(&mut db).await?;
    trace!(?procedures);
    view! {
        <h1>"Propagation Procedures"</h1>
        <ul>
            for p in procedures {
                <li><a href=(format!("/propagation/{}", p.id))>(p.name)</a></li>
            }
        </ul>
    }
}

#[page("/propagation/{propagation_id}")]
pub async fn get_propagation_details(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let propagation_id = path_param::<PropagationId>(cx)?;
    let procedure = PropagationProcedure::filter_by_id(propagation_id)
        .include(PropagationProcedure::fields().citations())
        .one()
        .exec(&mut db)
        .await?;
    // including the Taxa in the PropagationProcedure query is very slow, so fetch them separately
    let taxa = Taxon::filter(
        Taxon::fields().propagation_procedures().any(
            TaxonPropagationProcedure::fields()
                .propagation_id()
                .eq(propagation_id),
        ),
    )
    .include(
        Taxon::fields().propagation_procedures().filter(
            TaxonPropagationProcedure::fields()
                .propagation_id()
                .eq(propagation_id),
        ),
    )
    .exec(&mut db)
    .await?;
    trace!(?procedure);
    view! {
        <h1>(&procedure.name)</h1>
        <dt>"ID"</dt>
        <dd>(procedure.id)</dd>
        <dt>"Name"</dt>
        <dd>(procedure.name)</dd>
        <dt>"Type"</dt>
        <dd>(procedure.r#type.to_string())</dd>
        <dt>"Notes"</dt>
        <dd>(procedure.notes.as_deref().unwrap_or_default())</dd>
        <dt>"Instructions"</dt>
        <dd>(procedure.instructions)</dd>
        <dt>"Taxa"</dt>
        <dd>
            if !taxa.is_empty() {
                <table>
                    <tr>
                        <th>"ID"</th>
                        <th>"Name"</th>
                    </tr>
                    for taxon in taxa {
                        <tr>
                            <td>(taxon.id)</td>
                            <td>
                                <span class="latin">
                                    <a href=(taxon.path())>
                                        (&taxon.complete_name)
                                    </a>
                                </span>
                            </td>
                        </tr>
                    }
                </table>
            } else {
                "None"
            }
        </dd>
        <dt>"Citations"</dt>
        <dd>
            if !procedure.citations.get().is_empty() {
                citation_list(citations: procedure.citations.get().iter().collect())
            } else {
                "None"
            }
        </dd>
    }
}
