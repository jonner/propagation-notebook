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
    components::{
        self, Breadcrumb, breadcrumbs, citation_list, pagination_control, taxon_regional_table,
    },
    util::{CleaningId, ModifyOffset, PageState, Path, PropagationId, TaxonId, db},
};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultsFormat {
    List,
    Grid,
}

#[derive(Debug, Clone, serde::Serialize)]
#[query_params(error = bad_request)]
pub struct TaxaListParams {
    pub offset: Option<usize>,
    pub q: Option<String>,
    pub fmt: Option<ResultsFormat>,
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
    .include(Taxon::fields().photo())
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
        <form method="get" class="mb-6">
            <input
                type="text"
                name="q"
                placeholder="Search for a taxon"
                value=(params.q.as_deref().unwrap_or_default())
                class="me-2"
            >
            <button type="submit">"Search"</button>
        </form>
        match params.fmt {
            Some(ResultsFormat::Grid) => {
                <div
                    class="grid items-center gap-6 grid-cols-2 sm:grid-cols-5 xl:grid-cols-10"
                >
                    for taxon in taxa.iter() {
                        <div class="p-4 bg-jaggery/5 rounded h-full">
                            <a
                                href=(taxon.path())
                                class="flex flex-col items-center text-center"
                            >
                                if let Some(photo) = taxon.photo.get() {
                                    if let Some(url) = photo.square_url.as_ref() {
                                        <img class="block" src=(url)>
                                    }
                                }
                                <div>(&taxon.complete_name)</div>
                            </a>
                        </div>
                    }
                </div>
            }
            _ => {
                <ul>
                    for taxon in taxa.iter() {
                        <li>
                            <span class="latin">
                                <a href=(taxon.path())>(&taxon.complete_name)</a>
                            </span>
                        </li>
                    }
                </ul>
            }
        }
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
        .include(Taxon::fields().photo())
        .one()
        .exec(&mut db)
        .await?;
    trace!(?taxon);
    // for now, just manually traverse the ancestry. Maybe eventually use a CTE or something
    let mut ancestors = Vec::default();
    let mut next_parent = taxon.parent_id;
    while let Some(id) = next_parent {
        let t = Taxon::filter_by_id(id).one().exec(&mut db).await?;
        next_parent = t.parent_id;
        ancestors.push(Breadcrumb {
            url: Some(t.path()),
            text: t.complete_name,
        });
    }

    view! {
        breadcrumbs(items: ancestors, ellipsize: Some(2))
        <h1>
            <span class="latin">(&taxon.complete_name)</span>
            " ("
            (taxon.rank.to_string())
            ")"
        </h1>
        <div class="flex flex-col gap-4">
            if let Some(photo) = taxon.photo.get() {
                if let Some(medium_url) = photo.medium_url.as_ref() {
                    <figure>
                        <img
                            src=(medium_url)
                            alt=(&taxon.complete_name)
                            class="border shadow-xl"
                        >
                        <figcaption class="text-slate-500">
                            (photo.attribution.as_ref())
                        </figcaption>
                    </figure>
                }
            }
            if !taxon.vernaculars.get().is_empty() {
                <section>
                    <h2>"Common Name(s)"</h2>
                    <div>
                        <ul>
                            for cn in taxon.vernaculars.get() {
                                <li>(&cn.name)</li>
                            }
                        </ul>
                    </div>
                </section>
            }

            if !taxon.children.get().is_empty() {
                <section>
                    <h2>"Child taxa"</h2>
                    <div>
                        <ul>
                            for child in taxon.children.get() {
                                <li>
                                    <span class="latin">
                                        <a href=(child.path())>(&child.complete_name)</a>
                                    </span>
                                </li>
                            }
                        </ul>
                    </div>
                </section>
            }

            if let Some(collecting_data) = &taxon.collecting_data.get() {
                <section>
                    <h2>"Ripening"</h2>
                    <div>
                        (collecting_data
                            .ripening_indicators
                            .as_deref()
                            .unwrap_or_default())
                    </div>
                </section>
                <section>
                    <h2>"Harvesting Notes"</h2>
                    <div>
                        (collecting_data
                            .harvesting_notes
                            .as_deref()
                            .unwrap_or_default())
                    </div>
                </section>
                <section>
                    <h2>"Storage Conditions"</h2>
                    <div>
                        (collecting_data
                            .storage
                            .as_deref()
                            .unwrap_or_default())
                    </div>
                </section>
                <section>
                    <h2>"Storage Life"</h2>
                    <div>
                        (collecting_data
                            .storage_life
                            .as_deref()
                            .unwrap_or_default())
                    </div>
                </section>
            }
            if !taxon.cleaning_procedures.get().is_empty() {
                <section>
                    <h2>"Seed Cleaning"</h2>
                    <div>
                        <ul>
                            for procedure in taxon.cleaning_procedures.get() {
                                <li><a href=(procedure.path())>(&procedure.name)</a></li>
                            }
                        </ul>
                    </div>
                </section>
            }
            if !taxon.propagation_procedures.get().is_empty() {
                <section>
                    <h2>"Propagation Procedures"</h2>
                    <div>
                        <ul>
                            for tp in taxon.propagation_procedures.get() {
                                <li>
                                    <a href=(tp.path())>(&tp.propagation.get().name)</a>
                                </li>
                            }
                        </ul>
                    </div>
                </section>
            }
            if !taxon.regional_statuses.get().is_empty() {
                <section>
                    <h2>"Regions"</h2>
                    <div>
                        taxon_regional_table(regions: taxon.regional_statuses.get())
                    </div>
                </section>
            }
            if !taxon.notes.get().is_empty() {
                <section>
                    <h2>"Notes"</h2>
                    <div>
                        if !taxon.notes.get().is_empty() {
                            <table>
                                <thead>
                                    <tr>
                                        <th>"ID"</th>
                                        <th>"Name"</th>
                                    </tr>
                                </thead>
                                for note in taxon.notes.get() {
                                    <tr>
                                        <td>(note.id)</td>
                                        <td>(&note.text)</td>
                                    </tr>
                                }
                            </table>
                        }
                    </div>
                </section>
            }

            if !taxon.synonyms.get().is_empty() {
                <section>
                    <h2>"Synonyms"</h2>
                    <div>
                        <ul>
                            for syn in taxon.synonyms.get() {
                                <li>(&syn.complete_name)</li>
                            }
                        </ul>
                    </div>
                </section>
            }

            <section>
                <h2>"External Resources"</h2>
                <div>
                    <ul>
                        <li>
                            <a
                                href=(format!(
                                    "https://www.itis.gov/servlet/SingleRpt/SingleRpt?search_topic=TSN&search_value={}",
                                    taxon.itis_id
                                ))
                            >
                                "ITIS taxon info"
                            </a>
                        </li>
                        <li>
                            if let Some(id) = taxon.inaturalist_id {
                                <a
                                    href=(format!(
                                        "Https://www.inaturalist.org/taxa/{id}"
                                    ))
                                >
                                    "iNaturalist taxon info"
                                </a>
                            }
                        </li>
                    </ul>
                </div>
            </section>
        </div>
    }
}

#[page("/taxa/{taxon_id}/propagation/{propagation_id}")]
pub async fn propagation_details(cx: &Cx) -> topcoat::Result {
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
            .await?;
    trace!(?tp);
    let proc = tp.propagation.get();
    let taxon = tp.taxon.get();

    view! {
        <h1>
            <span>(&proc.name)</span>
            " for "
            <span class="latin">(&taxon.complete_name)</span>
        </h1>
        <h2>"General procedure information"</h2>
        <div class="ms-4">
            components::propagation_details(procedure: tp.propagation.get())
        </div>
        <h2>"Taxon-specific information"</h2>
        <div class="ms-4">
            <h3>"Confidence"</h3>
            <div>(tp.confidence.map(|v| v.to_string()).unwrap_or_default())</div>
            <h3>"Taxon-specific notes"</h3>
            <div>(tp.notes.as_deref().unwrap_or_default())</div>
            <h3>"Citations"</h3>
            <div>
                citation_list(
                    citations: tp
                        .citation_links
                        .get()
                        .iter()
                        .map(|cl| cl.citation.get())
                        .collect()
                )
            </div>
        </div>
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
    view! {
        <h1>
            <span>(&proc.name)</span>
            " for "
            <span class="latin">(&taxon.complete_name)</span>
        </h1>
        <dt>"Taxon"</dt>
        <dd>
            <span class="latin"><a href=(taxon.path())>(&taxon.complete_name)</a></span>
        </dd>
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
