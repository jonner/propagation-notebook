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

use crate::{components::propagation_details, taxa, util::db};

#[page("/propagation")]
pub async fn get_propagation_list(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let procedures = PropagationProcedure::all().exec(&mut db).await?;
    trace!(?procedures);
    view! {
        <h1>"Propagation Procedures"</h1>
        <ul>
            for p in procedures {
                <li>
                    <a href=(href!(get_propagation_details, PropagationId(p.id)))>
                        (p.name)
                    </a>
                </li>
            }
        </ul>
    }
}

path_param!(propagation_id: u64, error= bad_request);

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
        propagation_details(procedure: &procedure)
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
