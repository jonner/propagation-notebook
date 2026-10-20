use libpropagation::{citation::Citation, taxonomy::TaxonPropagationProcedure};
use topcoat::{
    context::Cx,
    router::{error::RouterErrorExt, href, page, path_param},
    view::{View, attributes, view},
};

use crate::{
    components::{badge::badge, breadcrumb::*, citations::citation_list},
    context::db,
    taxa::{self, TaxonId},
};

path_param!(pub propagation_id: u64, error = bad_request);

#[page("/taxa/{taxon_id}/propagation/{propagation_id}")]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let mut db = db(cx);
    let taxon_id = path_param::<TaxonId>(cx)?;
    let propagation_id = path_param::<PropagationId>(cx)?;
    let tp =
        TaxonPropagationProcedure::filter_by_taxon_id_and_propagation_id(taxon_id, propagation_id)
            .include(
                TaxonPropagationProcedure::fields()
                    .propagation()
                    .citations(),
            )
            .include(TaxonPropagationProcedure::fields().taxon())
            .include(
                TaxonPropagationProcedure::fields()
                    .citation_links()
                    .citation(),
            )
            .one()
            .exec(&mut db)
            .await
            .ok_or_not_found()?;

    Ok(view! {
        let procedure = tp.propagation.get();
        let taxon = tp.taxon.get();
        <div class="flex flex-col gap-4">
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(taxa::details, TaxonId(taxon.id))) },
                                (&taxon.complete_name)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_page(
                                (format!("Propagation Procedure {}", tp.propagation_id))
                            )
                        )
                    )
                )
                <h1>(&procedure.name)</h1>
                <div>badge((procedure.r#type.to_string()))</div>
            </hgroup>
            <section>
                <h2>"Instructions"</h2>
                <div class="text-2xl">(&procedure.instructions)</div>
            </section>
            <section>
                <h2>"Confidence"</h2>
                <div>
                    (tp.confidence.map(|v| v.to_string()).unwrap_or("Unknown".into()))
                </div>
            </section>
            <section>
                <h2>"Notes"</h2>
                <div>(procedure.notes.as_deref().unwrap_or_default())</div>
                if let Some(taxon_notes) = tp.notes {
                    <div>(taxon_notes)</div>
                }
            </section>
            <section>
                <h2>"Citations"</h2>
                <div>
                    let citations: Vec<&Citation> = tp
                        .citation_links
                        .get()
                        .iter()
                        .map(|cl| cl.citation.get())
                        .chain(procedure.citations.get().iter())
                        .collect();
                    if !citations.is_empty() {
                        citation_list(citations: citations)
                    }
                </div>
            </section>
        </div>
    })
}
