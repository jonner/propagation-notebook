use libpropagation::{
    propagation::PropagationProcedure,
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use topcoat::{
    context::Cx,
    router::{href, page, path_param},
    view::view,
};
use tracing::trace;

use crate::{components::citations::citation_list, taxa, util::db};

#[page("/propagation")]
pub async fn list(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let procedures = PropagationProcedure::all().exec(&mut db).await?;
    trace!(?procedures);
    view! {
        <h1>"Propagation Procedures"</h1>
        <ul>
            for p in procedures {
                <li><a href=(href!(details, PropagationId(p.id)))>(p.name)</a></li>
            }
        </ul>
    }
}

path_param!(propagation_id: u64, error= bad_request);

#[page("/propagation/{propagation_id}")]
pub async fn details(cx: &Cx) -> topcoat::Result {
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
        <h3>"Type"</h3>
        <div>(procedure.r#type.to_string())</div>
        <h3>"Notes"</h3>
        <div>(procedure.notes.as_deref().unwrap_or_default())</div>
        <h3>"Instructions"</h3>
        <div>(&procedure.instructions)</div>
        <h3>"Citations"</h3>
        <div>
            if !procedure.citations.get().is_empty() {
                citation_list(citations: procedure.citations.get().iter().collect())
            } else {
                "None"
            }
        </div>
        <div>
            <h2>"Taxa"</h2>
            if !taxa.is_empty() {
                <table>
                    <thead>
                        <tr>
                            <th>"ID"</th>
                            <th>"Name"</th>
                        </tr>
                    </thead>
                    for taxon in taxa {
                        <tr>
                            <td>(taxon.id)</td>
                            <td>
                                <span class="latin">
                                    <a href=(href!(taxa::details, taxa::TaxonId(taxon.id)))>
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
        </div>
    }
}
