use libpropagation::{
    collecting::CleaningProcedure,
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use topcoat::{
    context::Cx,
    router::{page, path_param, query_params},
    view::{attributes, view},
};
use tracing::trace;

use crate::{
    components::{
        self, citation_list, conservation_status_badge, harvest_timeline, origin_badge,
        pagination_control,
    },
    util::{CleaningId, ModifyOffset, PageState, Path, PropagationId, TaxonId, db},
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
        <ul>
            for taxon in taxa.iter() {
                <li>
                    <span class="latin">
                        <a href=(taxon.path())>(&taxon.complete_name)</a>
                    </span>
                </li>
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
        .include(Taxon::fields().photo())
        .one()
        .exec(&mut db)
        .await?;
    trace!(?taxon);

    view! {
        <h1><span class="latin">(&taxon.complete_name)</span></h1>
        if let Some(photo) = taxon.photo.get() {
            if let Some(medium_url) = photo.medium_url.as_ref() {
                <figure class="mb-4">
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
                Some(p) => <span class="latin">
                    <a href=(p.path())>(&p.complete_name)</a>
                </span>,
                None => "",
            }
        </dd>

        <dt>"Child taxa"</dt>
        <dd>
            <ul>
                for child in taxon.children.get() {
                    <li>
                        <span class="latin">
                            <a href=(child.path())>(&child.complete_name)</a>
                        </span>
                    </li>
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
                        <th>"Name"</th>
                        <th>"Origin"</th>
                        <th>"Status"</th>
                        <th>"Fruiting Window"</th>
                    </tr>
                    for rs in taxon.regional_statuses.get() {
                        <tr>
                            <td><a href=(rs.path())>(&rs.region.get().name)</a></td>
                            <td>
                                if let Some(origin) = rs.origin {
                                    origin_badge(origin: origin)
                                }
                            </td>
                            <td>
                                if let Some(status) = rs.conservation_status {
                                    conservation_status_badge(status: status)
                                }
                            </td>
                            <td>
                                <div class="flex items-center gap-x-6">
                                    harvest_timeline(
                                        window: &rs.harvest_window,
                                        attrs: attributes! { class="w-120" }
                                    )
                                    (rs.harvest_window.to_string())
                                </div>
                            </td>
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
        <dt>"External Resources"</dt>
        <dd>
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
