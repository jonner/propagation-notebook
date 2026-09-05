use libpropagation::{
    citation::Citation,
    cleaning::CleaningProcedure,
    region::{Origin, Region, RegionalTaxonStatus},
    taxonomy::{Taxon, TaxonHierarchy, TaxonNote, TaxonPropagationProcedure},
};
use topcoat::{
    context::Cx,
    icon::icon,
    router::{
        error::{RouterErrorExt, redirect},
        href, page, path_param, query_params,
    },
    view::{attributes, view},
};
use tracing::trace;

use crate::{
    components::{
        badge::{BadgeVariant, badge, origin_badge},
        breadcrumb::*,
        button::button,
        citations::citation_list,
        harvest::taxon_regional_table,
        input::input,
        pagination::pagination_control,
        taxa_grid, taxa_grid_item, taxon_icon,
    },
    mdi,
    util::{ModifyOffset, PER_PAGE, PageState, db},
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

#[derive(Debug, Clone, serde::Serialize, Default)]
#[query_params(error = bad_request)]
pub struct TaxaListParams {
    pub offset: Option<usize>,
    pub parent: Option<u64>,
    pub region: Option<u64>,
    pub fmt: Option<ResultsFormat>,
}

impl ModifyOffset for TaxaListParams {
    fn modify_offset(&mut self, new_offset: usize) {
        self.offset = Some(new_offset);
    }
}

#[page("/taxa")]
pub(crate) async fn taxonomy(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let params = query_params::<TaxaListParams>(cx)?;
    let parent_id = match params.parent {
        Some(parent) => parent,
        None => {
            Taxon::get_by_complete_name(&mut db, "Tracheophyta")
                .await?
                .id
        }
    };
    let mut filter = Taxon::fields().parent_id().eq(parent_id);
    let mut extra_includes = None;
    if let Some(region) = params.region {
        filter = filter.and(
            Taxon::fields().descendant_links().any(
                TaxonHierarchy::fields()
                    .descendant()
                    .regional_statuses()
                    .any(RegionalTaxonStatus::fields().region_id().eq(region)),
            ),
        );
        extra_includes = Some(
            Taxon::fields()
                .regional_statuses()
                .filter(RegionalTaxonStatus::fields().region_id().eq(region)),
        );
    }

    let mut query = Taxon::filter(filter).include(Taxon::fields().photo());
    if let Some(extra_includes) = extra_includes {
        query = query.include(extra_includes);
    }
    query = query.order_by((
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
    if taxa.is_empty() {
        return Err(redirect(href!(details, TaxonId(parent_id)).resolve(cx)).into());
    }
    let ancestors = TaxonHierarchy::filter(TaxonHierarchy::fields().descendant_id().eq(parent_id))
        .include(TaxonHierarchy::fields().ancestor())
        .order_by(TaxonHierarchy::fields().depth().desc())
        .exec(&mut db)
        .await?;
    let ancestor_taxa = ancestors
        .iter()
        .map(|l| l.ancestor.get())
        .collect::<Vec<_>>();
    let region = if let Some(region_id) = params.region {
        Some(Region::get_by_id(&mut db, region_id).await?)
    } else {
        None
    };

    #[derive(Debug)]
    struct TaxonData {
        id: u64,
        name: String,
        origin: Option<Origin>,
        photo: Option<String>,
    }
    let taxa = taxa
        .into_iter()
        .map(|t| TaxonData {
            id: t.id,
            name: t.complete_name,
            origin: if !t.regional_statuses.is_unloaded() {
                t.regional_statuses
                    .get()
                    .iter()
                    .next()
                    .and_then(|rts| rts.origin)
            } else {
                None
            },
            photo: t.photo.into_inner().and_then(|p| p.square_url),
        })
        .collect::<Vec<_>>();

    view! {
        <div class="flex flex-col gap-3">
            <form method="get" action=(href!(search)) class="flex my-6 w-full">
                input(
                    attrs: attributes! {
                        type="text"
                        name="q"
                        placeholder="Search for a taxon"
                        class="me-2 flex-grow"
                    }
                )
                button(attrs: attributes! { type="submit" }, "Search")
            </form>
            <hgroup>
                <h1>"Taxonomy Explorer"</h1>
                if let Some(region) = region {
                    <div class="text-muted-foreground">
                        "Filters: "
                        badge(
                            variant: BadgeVariant::Secondary,
                            (format!("Region: '{}'", region.name))
                            <a
                                href=(href!(taxonomy)
                                    .query(TaxaListParams {
                                        region: None,
                                        offset: None,
                                        parent: params.parent,
                                        fmt: params.fmt,
                                    }))
                                class="p-2"
                            >
                                icon(data: mdi::CLEAR_BOLD, label: "Clear filter")
                            </a>
                        )
                    </div>
                }
                <div class="my-3">
                    ancestor_breadcrumbs(items: &ancestor_taxa, link_final: false)
                </div>
            </hgroup>
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
            match params.fmt {
                Some(ResultsFormat::List) => {
                    <ul class="contents">
                        for taxon in taxa.iter() {
                            <li>
                                <span class="latin">
                                    <a
                                        href=(href!(taxonomy)
                                            .query(TaxaListParams {
                                                parent: Some(taxon.id),
                                                fmt: params.fmt,
                                                offset: None,
                                                region: params.region,
                                            }))
                                    >
                                        (&taxon.name)
                                    </a>
                                    if let Some(origin) = taxon.origin {
                                        origin_badge(origin: origin)
                                    }
                                    <a href=(href!(details, TaxonId(taxon.id)))>
                                        icon(data: mdi::INFORMATION, label: "Information")
                                    </a>
                                </span>
                            </li>
                        }
                    </ul>
                }
                _ => {
                    taxa_grid(
                        for taxon in taxa.iter() {
                            taxa_grid_item(
                                <a
                                    href=(href!(taxonomy)
                                        .query(TaxaListParams {
                                            parent: Some(taxon.id),
                                            fmt: params.fmt,
                                            offset: None,
                                            region: params.region,
                                        }))
                                >
                                    taxon_icon(url: taxon.photo.as_ref())
                                </a>
                                <span>
                                    <a
                                        href=(href!(taxonomy)
                                            .query(TaxaListParams {
                                                parent: Some(taxon.id),
                                                fmt: params.fmt,
                                                offset: None,
                                                region: params.region,
                                            }))
                                    >
                                        (&taxon.name)
                                    </a>
                                    if let Some(origin) = taxon.origin {
                                        origin_badge(
                                            origin: origin,
                                            attrs: attributes! { class="ms-1" }
                                        )
                                    }
                                </span>
                                <a class="p-3" href=(href!(details, TaxonId(taxon.id)))>
                                    icon(data: mdi::INFORMATION_OUTLINE, label: "Information")
                                </a>
                            )
                        }
                    )
                }
            }
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state, params: params)
            }
        </div>
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[query_params(error = bad_request)]
pub struct TaxaSearchParams {
    pub offset: Option<usize>,
    pub q: Option<String>,
    pub fmt: Option<ResultsFormat>,
}

impl ModifyOffset for TaxaSearchParams {
    fn modify_offset(&mut self, new_offset: usize) {
        self.offset = Some(new_offset);
    }
}

#[page("/taxa/search")]
pub(crate) async fn search(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let params = query_params::<TaxaSearchParams>(cx)?;
    let (page_state, taxa) = match params.q.as_ref() {
        Some(q) => {
            let query = Taxon::filter(Taxon::search_filter(q))
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
            (page_state, Some(taxa))
        }
        None => (
            PageState {
                per_page: PER_PAGE,
                offset: 0,
                total: 0,
            },
            None,
        ),
    };
    view! {
        <div class="flex flex-col gap-3">
            <div>
                <h1>"Taxon Search"</h1>
                <form method="get" class="flex my-6 w-full">
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
            </div>
            if let Some(taxa) = taxa {
                <section>
                    <h3>"Results"</h3>
                    if taxa.is_empty() && let Some(q) = params.q.as_ref() {
                        <div>(format!("No taxa found for search term '{q}'"))</div>
                    }
                    if page_state.total_pages() > 1 {
                        pagination_control(state: &page_state, params: params)
                    }
                    <div class="my-3">
                        match params.fmt {
                            Some(ResultsFormat::Grid) => {
                                taxa_grid(
                                    for taxon in taxa.iter() {
                                        taxa_grid_item(
                                            <a
                                                href=(href!(details, TaxonId(taxon.id)))
                                                class="flex flex-col items-center text-center"
                                            >
                                                taxon_icon(
                                                    url: taxon
                                                        .photo
                                                        .get()
                                                        .as_ref()
                                                        .and_then(|p| p.square_url.as_ref())
                                                )
                                                <div>(&taxon.complete_name)</div>
                                            </a>
                                        )
                                    }
                                )
                            }
                            _ => {
                                <ul class="px-2">
                                    for taxon in taxa.iter() {
                                        <li class="py-1">
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
                    </div>
                    if page_state.total_pages() > 1 {
                        pagination_control(state: &page_state, params: params)
                    }
                </section>
            }
        </div>
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
        .await
        .ok_or_not_found()?;
    trace!(?taxon);
    let ancestors = taxon
        .ancestor_links
        .get()
        .iter()
        .rev()
        .filter_map(|l| {
            if l.depth == 0 {
                None
            } else {
                Some(l.ancestor.get())
            }
        })
        .collect::<Vec<_>>();

    view! {
        ancestor_breadcrumbs(items: &ancestors, link_final: true)
        <h1 class="flex items-center">
            <span class="latin">(&taxon.complete_name)</span>
            badge(
                variant: BadgeVariant::Secondary,
                attrs: attributes! { class="mx-3" },
                (taxon.rank.to_string())
            )
        </h1>
        <div class="flex flex-col gap-4">
            if let Some(photo) = taxon.photo.get() {
                if let Some(photo_url) = photo.medium_url.as_ref() {
                    <a href=(photo.original_url.as_ref())>
                        <figure class="taxon-photo">
                            <img src=(photo_url) alt=(&taxon.complete_name)>
                            <figcaption>(photo.attribution.as_ref())</figcaption>
                        </figure>
                    </a>
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

            if !taxon.regional_statuses.get().is_empty() {
                <section>
                    <h2>"Regions"</h2>
                    <div>
                        taxon_regional_table(regions: taxon.regional_statuses.get())
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
                                        "https://www.inaturalist.org/taxa/{id}"
                                    ))
                                >
                                    "iNaturalist taxon info"
                                </a>
                            }
                        </li>
                        for resource in taxon.resources.get() {
                            <li><a href=(&resource.url)>(&resource.name)</a></li>
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
            .await
            .ok_or_not_found()?;
    trace!(?tp);
    let procedure = tp.propagation.get();
    let taxon = tp.taxon.get();

    view! {
        <div class="flex flex-col gap-4">
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(taxonomy)) },
                                "Taxonomy"
                            )
                            breadcrumb_separator()
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(details, TaxonId(taxon.id))) },
                                (&taxon.complete_name)
                            )
                            breadcrumb_separator()
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
                    (tp
                        .confidence
                        .map(|v| v.to_string())
                        .unwrap_or("Unknown".into()))
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
                    let citations: Vec < & Citation > = tp
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
    .await
    .ok_or_not_found()?;
    trace!(?proc);
    let taxon = proc.taxon.get();

    view! {
        <div class="flex flex-col gap-4">
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(taxonomy)) },
                                "Taxonomy"
                            )
                            breadcrumb_separator()
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(details, TaxonId(taxon.id))) },
                                (&taxon.complete_name)
                            )
                            breadcrumb_separator()
                            breadcrumb_page((format!("Cleaning Procedure {}", proc.id)))
                        )
                    )
                )
                <h1>
                    <span>(&proc.name)</span>
                    " for "
                    <span class="latin">(&taxon.complete_name)</span>
                </h1>
            </hgroup>
            <section>
                <h2>"Instructions"</h2>
                <div class="text-2xl">(proc.instructions)</div>
            </section>
            if let Some(notes) = proc.notes {
                <section>
                    <h2>"Additional Notes"</h2>
                    <div>(notes)</div>
                </section>
            }
            <section>
                <h2>"Citations"</h2>
                <div>
                    citation_list(
                        citations: proc
                            .citation_links
                            .get()
                            .iter()
                            .map(|cl| cl.citation.get())
                            .collect()
                    )
                </div>
            </section>
        </div>
    }
}

#[page("/taxa/{taxon_id}/note/{note_id}")]
pub async fn note_details(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let _taxon_id = path_param::<TaxonId>(cx)?;
    let note_id = path_param::<NoteId>(cx)?;
    let note = TaxonNote::filter_by_id(note_id)
        .include(TaxonNote::fields().taxon())
        .include(TaxonNote::fields().citation_links().citation())
        .one()
        .exec(&mut db)
        .await
        .ok_or_not_found()?;
    trace!(?note);
    let taxon = note.taxon.get();

    view! {
        <div class="flex flex-col gap-4">
            <section>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(taxonomy)) },
                                "Taxonomy"
                            )
                            breadcrumb_separator()
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(details, TaxonId(taxon.id))) },
                                (&taxon.complete_name)
                            )
                            breadcrumb_separator()
                            breadcrumb_page((format!("Note {}", note.id)))
                        )
                    )
                )
                <h1>(&note.title)</h1>
                <div class="text-2xl">(note.text)</div>
            </section>
            <section>
                citation_list(
                    citations: note
                        .citation_links
                        .get()
                        .iter()
                        .map(|cl| cl.citation.get())
                        .collect()
                )
            </section>
        </div>
    }
}
