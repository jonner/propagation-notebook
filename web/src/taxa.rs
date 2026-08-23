use libpropagation::{
    cleaning::CleaningProcedure,
    taxonomy::{Taxon, TaxonNote, TaxonPropagationProcedure},
};
use topcoat::{
    context::Cx,
    router::{href, page, path_param, query_params},
    view::{attributes, view},
};
use tracing::trace;

use crate::{
    components::{
        self, Breadcrumb, breadcrumbs, button::button, citations::citation_list,
        harvest::taxon_regional_table, input::input, pagination::pagination_control,
    },
    util::{ModifyOffset, PER_PAGE, PageState, db, enum_to_string},
};

path_param!(pub cleaning_id: u64, error = bad_request);
path_param!(pub taxon_id: u64, error = bad_request);
path_param!(pub region_id: u64, error = bad_request);
path_param!(pub propagation_id: u64, error = bad_request);
path_param!(pub note_id: u64, error = bad_request);

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
    let page_state = PageState::new(params.offset, PER_PAGE, total);
    let taxa = query
        .limit(page_state.per_page)
        .offset(page_state.offset)
        .exec(&mut db)
        .await?;
    trace!(?taxa);
    view! {
        <h1>"Taxon List"</h1>
        <form method="get" action="/taxa" class="flex my-6 w-full md:w-xl">
            input(
                attrs: attributes! {
                    type="text"
                    name="q"
                    placeholder="Search for a taxon"
                    value=(params.q.as_deref().unwrap_or_default())
                    class="me-2 flex-grow"
                }
            )
            button(attrs: attributes! { type="submit" }, "Search")
        </form>
        if page_state.total_pages() > 1 {
            pagination_control(state: &page_state, params: params)
        }
        match params.fmt {
            Some(ResultsFormat::Grid) => {
                <div
                    class="grid items-center gap-6 grid-cols-2 sm:grid-cols-5 xl:grid-cols-10"
                >
                    for taxon in taxa.iter() {
                        <div class="p-4 bg-jaggery/5 rounded h-full">
                            <a
                                href=(href!(details, TaxonId(taxon.id)))
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
                <ul class="contents">
                    for taxon in taxa.iter() {
                        <li>
                            <span class="latin">
                                <a href=(href!(details, TaxonId(taxon.id)))>
                                    (&taxon.complete_name)
                                </a>
                            </span>
                        </li>
                    }
                </ul>
            }
        }
        if page_state.total_pages() > 1 {
            pagination_control(state: &page_state, params: params)
        }
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
        .include(
            Taxon::fields()
                .children()
                .order_by(Taxon::fields().sequence().asc()),
        )
        .include(Taxon::fields().cleaning_procedures())
        .include(Taxon::fields().propagation_procedures().propagation())
        .include(Taxon::fields().regional_statuses().region())
        .include(Taxon::fields().notes())
        .include(Taxon::fields().photo())
        .include(Taxon::fields().ancestor_links().ancestor())
        .include(Taxon::fields().resources())
        .one()
        .exec(&mut db)
        .await?;
    trace!(?taxon);
    let ancestors = taxon
        .ancestor_links
        .get()
        .iter()
        .filter_map(|l| {
            if l.depth == 0 {
                None
            } else {
                Some(Breadcrumb {
                    url: Some(href!(details, TaxonId(l.ancestor.get().id)).resolve(cx)),
                    text: l.ancestor.get().complete_name.clone(),
                })
            }
        })
        .rev()
        .collect::<Vec<_>>();

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
                                        <a href=(href!(details, TaxonId(child.id)))>
                                            (&child.complete_name)
                                        </a>
                                    </span>
                                </li>
                            }
                        </ul>
                    </div>
                </section>
            }

            if !taxon.cleaning_procedures.get().is_empty() {
                <section>
                    <h2>"Seed Cleaning"</h2>
                    <div>
                        <ul>
                            for procedure in taxon.cleaning_procedures.get() {
                                <li>
                                    <a
                                        href=(href!(cleaning_details, TaxonId(procedure.taxon_id), CleaningId(procedure.id)))
                                    >
                                        (&procedure.name)
                                    </a>
                                </li>
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
                                    <a
                                        href=(href!(propagation_details, TaxonId(tp.taxon_id), PropagationId(tp.propagation_id)))
                                    >
                                        (&tp.propagation.get().name)
                                    </a>
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
                    <ul>
                        for note in taxon.notes.get() {
                            <li>
                                <a
                                    href=(href!(note_details, TaxonId(taxon.id), NoteId(note.id)))
                                >
                                    (&note.title)
                                </a>
                            </li>
                        }
                    </ul>
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
                                        "https://www.inaturalist.org/taxa/{id}"
                                    ))
                                >
                                    "iNaturalist taxon info"
                                </a>
                            }
                        </li>
                        for resource in taxon.resources.get() {
                            <a href=(&resource.url)>(&resource.name)</a>
                        }
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
            <span class="latin">
                <a href=(href!(details, TaxonId(taxon.id)))>(&taxon.complete_name)</a>
            </span>
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

#[page("/taxa/{taxon_id}/note/{note_id}")]
pub async fn note_details(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let taxon_id = path_param::<TaxonId>(cx)?;
    let note_id = path_param::<NoteId>(cx)?;
    let note = TaxonNote::filter_by_id(note_id)
        .include(TaxonNote::fields().taxon())
        .include(TaxonNote::fields().citation_links().citation())
        .one()
        .exec(&mut db)
        .await?;
    trace!(?note);
    let taxon = note.taxon.get();
    view! {
        <h1>
            <span>(&note.title)</span>
            " for "
            <span class="latin">(&taxon.complete_name)</span>
        </h1>
        <dt>"Taxon"</dt>
        <dd>
            <span class="latin">
                <a href=(href!(details, TaxonId(taxon.id)))>(&taxon.complete_name)</a>
            </span>
        </dd>
        <dt>"Body"</dt>
        <dd>(note.text)</dd>
        <dt>"Citations"</dt>
        <dd>
            citation_list(
                citations: note
                    .citation_links
                    .get()
                    .iter()
                    .map(|cl| cl.citation.get())
                    .collect()
            )
        </dd>
    }
}
