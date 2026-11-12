use std::{cmp::Ordering, collections::BTreeMap};

use libpropagation::{
    auth::PermissionCode,
    region::{ConservationStatus, Origin, Region, RegionalHarvestWindow, RegionalTaxonStatus},
    taxonomy::{Taxon, TaxonHierarchy, TaxonNote, TaxonNoteCategory},
};
use serde::Serialize;
use topcoat::{
    context::{Cx, memoize},
    icon::icon,
    metadata::Metadata,
    router::{
        error::{RouterErrorExt, redirect},
        href, page, path_param, query_params,
    },
    runtime::{Event, Signal, procedure, signal},
    view::{
        AttributeValueViewParts, Attributes, Child, View, attributes, class, component,
        error_boundary, suspense, view,
    },
};
use tracing::{debug, warn};

use crate::{
    Subtitle,
    actions::{Action, ActionItemType},
    assets::mdi,
    citation,
    components::{
        action_menu::*,
        badge::{BadgeVariant, badge},
        breadcrumb::*,
        button::button,
        harvest::taxon_regional_table,
        input::input,
        pn::{origin_badge, pagination_control, taxon_icon},
    },
    context::{current_user, db, require_user_with_permission},
    util::{PER_PAGE, PageState},
};

mod cleaning;
mod note;
mod propagation;

path_param!(pub taxon_id: u64 else bad_request);
path_param!(pub region_id: u64 else bad_request);

#[component]
pub async fn taxa_grid(
    cx: &Cx,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div class=(class!("results-grid", attrs.remove("class"))) (attrs)>(child)</div>
    })
}
#[component]
pub async fn taxa_grid_item(
    cx: &Cx,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class!(
                "p-6 h-full flex flex-col items-center text-center justify-center",
                attrs.remove("class"),
            ))
            (attrs)
        >
            (child)
        </div>
    })
}

#[component]
pub async fn ancestor_breadcrumbs<L, P>(
    cx: &Cx,
    items: &[&Taxon],
    link_fn: L,
    #[default] link_final: bool,
    #[default(Some(2))] ellipsize: Option<usize>,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View>
where
    L: Fn(&Taxon) -> P + Send + Sync,
    P: AttributeValueViewParts,
{
    let mut item_iter = items.iter();
    let last = item_iter.next_back();
    let root = item_iter.next();
    let middle_items: Vec<_> = item_iter.collect();

    let total_len = items.len();
    let do_ellipsize = ellipsize.map(|limit| total_len > limit).unwrap_or(false);

    let middle_to_render = if do_ellipsize {
        if let Some(limit) = ellipsize {
            middle_items
                .iter()
                .rev()
                .take(limit.saturating_sub(1))
                .rev()
                .copied()
                .collect()
        } else {
            middle_items
        }
    } else {
        middle_items
    };

    Ok(view! {
        breadcrumb(
            breadcrumb_list(
                if let Some(taxon) = root {
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! { href=(link_fn(taxon)) },
                            (&taxon.complete_name)
                        )
                    )
                }
                if do_ellipsize {
                    breadcrumb_separator()
                    breadcrumb_ellipsis()
                }
                for taxon in middle_to_render {
                    breadcrumb_separator()
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! { href=(link_fn(taxon)) },
                            (&taxon.complete_name)
                        )
                    )
                }

                if let Some(taxon) = last {
                    if root.is_some() {
                        breadcrumb_separator()
                    }
                    breadcrumb_item(
                        if link_final {
                            breadcrumb_link(
                                attrs: attributes! { href=(link_fn(taxon)) },
                                (&taxon.complete_name)
                            )
                        } else {
                            breadcrumb_page(
                                (&taxon.complete_name)
                                badge(
                                    variant: BadgeVariant::Secondary,
                                    attrs: attributes! { class="mx-3" },
                                    (taxon.rank.to_string())
                                )
                            )
                        }
                    )
                }
                (child)
            )
        )
    })
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultsFormat {
    List,
    Grid,
}

#[derive(Debug, Clone, Default, Serialize)]
#[query_params(error = bad_request)]
pub struct TaxaListParams {
    pub offset: Option<usize>,
    pub parent: Option<u64>,
    pub region: Option<u64>,
    pub fmt: Option<ResultsFormat>,
}

#[page("/taxa/explore", with(Metadata::new().subtitle("Taxonomy Explorer")))]
pub(crate) async fn explore(cx: &Cx) -> topcoat::Result<impl View> {
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
        return Err(redirect(href!(details, taxon_id: parent_id).resolve(cx)).into());
    }
    let ancestors = TaxonHierarchy::filter(TaxonHierarchy::fields().descendant_id().eq(parent_id))
        .include(TaxonHierarchy::fields().ancestor())
        .order_by(TaxonHierarchy::fields().depth().desc())
        .exec(&mut db)
        .await?;
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

    Ok(view! {
        let ancestor_taxa = ancestors
            .iter()
            .map(|l| l.ancestor.get())
            .collect::<Vec<_>>();
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
                                href=(href!(explore).query(
                                    TaxaListParams {
                                        region: None,
                                        offset: None,
                                        parent: params.parent,
                                        fmt: params.fmt,
                                    },
                                ))
                                class="p-2"
                            >
                                icon(data: mdi::CLEAR_BOLD, label: "Clear filter")
                            </a>
                        )
                    </div>
                }
                <div class="my-3">
                    ancestor_breadcrumbs(
                        items: &ancestor_taxa,
                        link_fn: |taxon| {
                            href!(explore).query(
                                TaxaListParams {
                                    offset: None,
                                    parent: Some(taxon.id),
                                    fmt: params.fmt,
                                    region: params.region,
                                },
                            )
                        },
                        link_final: false
                    )
                </div>
            </hgroup>
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state)
            }
            match params.fmt {
                Some(ResultsFormat::List) => {
                    <ul class="contents">
                        for taxon in taxa.iter() {
                            <li>
                                <span class="latin">
                                    <a
                                        href=(href!(explore).query(
                                            TaxaListParams {
                                                parent: Some(taxon.id),
                                                fmt: params.fmt,
                                                offset: None,
                                                region: params.region,
                                            },
                                        ))
                                    >
                                        (&taxon.name)
                                    </a>
                                    if let Some(origin) = taxon.origin {
                                        origin_badge(origin: origin)
                                    }
                                    <a href=(href!(details, taxon_id: taxon.id))>
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
                                    href=(href!(explore).query(
                                        TaxaListParams {
                                            parent: Some(taxon.id),
                                            fmt: params.fmt,
                                            offset: None,
                                            region: params.region,
                                        },
                                    ))
                                >
                                    taxon_icon(url: taxon.photo.as_ref())
                                </a>
                                <span>
                                    <a
                                        href=(href!(explore).query(
                                            TaxaListParams {
                                                parent: Some(taxon.id),
                                                fmt: params.fmt,
                                                offset: None,
                                                region: params.region,
                                            },
                                        ))
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
                                <a class="p-3" href=(href!(details, taxon_id: taxon.id))>
                                    icon(data: mdi::INFORMATION_OUTLINE, label: "Information")
                                </a>
                            )
                        }
                    )
                }
            }
            if page_state.total_pages() > 1 {
                pagination_control(state: &page_state)
            }
        </div>
    })
}

#[derive(Debug, Clone)]
#[query_params(error = bad_request)]
pub struct TaxaSearchParams {
    pub offset: Option<usize>,
    pub q: Option<String>,
    pub fmt: Option<ResultsFormat>,
}

#[page("/taxa/search", with(Metadata::new().subtitle("Taxon Search")))]
pub(crate) async fn search(cx: &Cx) -> topcoat::Result<impl View> {
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
    Ok(view! {
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
                        pagination_control(state: &page_state)
                    }
                    <div class="my-3">
                        match params.fmt {
                            Some(ResultsFormat::Grid) => {
                                taxa_grid(
                                    for taxon in taxa.iter() {
                                        taxa_grid_item(
                                            <a
                                                href=(href!(details, taxon_id: taxon.id))
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
                                                <a href=(href!(details, taxon_id: taxon.id))>
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
                        pagination_control(state: &page_state)
                    }
                </section>
            }
        </div>
    })
}

#[memoize(as_ref)]
pub async fn taxon(cx: &Cx) -> topcoat::Result<Taxon> {
    let id = taxon_id(cx)?;
    Ok(Taxon::filter_by_id(id)
        .include(Taxon::fields().photo())
        .include(Taxon::fields().ancestor_links().ancestor())
        .one()
        .exec(&mut db(cx))
        .await
        .ok_or_not_found()?)
}

#[page(
    "/taxa/{taxon_id}/photo/default",
    with(Metadata::from_fn(
        async |cx| {
            let taxon = taxon(cx).await?;
            Ok(Metadata::new().subtitle(format!("{} photo", taxon.complete_name)))
        },
    ))
)]
pub async fn default_photo(cx: &Cx) -> topcoat::Result<impl View> {
    let taxon = taxon(cx).await?;
    let photo = taxon.photo.get().as_ref().ok_or_not_found()?;
    let photo_url = photo.original_url.as_ref().ok_or_not_found()?;

    Ok(view! {
        let ancestors = taxon
            .ancestor_links
            .get()
            .iter()
            .rev()
            .map(|l| l.ancestor.get())
            .collect::<Vec<_>>();
        ancestor_breadcrumbs(
            items: &ancestors,
            link_fn: |taxon| { href!(crate::taxa::details, taxon_id: taxon.id) },
            ellipsize: Some(1),
            link_final: true,
            breadcrumb_separator()
            breadcrumb_page("Default photo")
        )
        <h1 class="flex items-center">
            <span class="latin">(&taxon.complete_name)</span>
            badge(
                variant: BadgeVariant::Secondary,
                attrs: attributes! { class="mx-3" },
                (taxon.rank.to_string())
            )
        </h1>
        <div class="flex flex-col gap-4">
            <figure class="taxon-photo">
                <img src=(photo_url) alt=(&taxon.complete_name)>
                <figcaption>(photo.attribution.as_ref())</figcaption>
            </figure>
        </div>
    })
}

#[derive(Debug, Clone)]
pub struct RegionHarvestWindowSummary {
    pub harvest_window: RegionalHarvestWindow,
    pub origin: Option<Origin>,
    pub conservation_status: Option<ConservationStatus>,
}

impl From<&RegionalTaxonStatus> for RegionHarvestWindowSummary {
    fn from(value: &RegionalTaxonStatus) -> Self {
        value.clone().into()
    }
}

impl From<RegionalTaxonStatus> for RegionHarvestWindowSummary {
    fn from(value: RegionalTaxonStatus) -> Self {
        Self {
            harvest_window: value.harvest_window,
            origin: value.origin,
            conservation_status: value.conservation_status,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ChildRegionError {
    #[error("Too many children to display child regions")]
    TooManyChildren,
}

// Look up the regional statuses for all descendant taxa of this taxon and
// return regional status information that encapsulates all  descendant taxa
#[component]
pub async fn descendant_region_statuses(cx: &Cx, parent_id: u64) -> topcoat::Result<impl View> {
    let mut db = db(cx);
    let filter = RegionalTaxonStatus::filter(
        RegionalTaxonStatus::fields()
            .taxon()
            .ancestors()
            .any(Taxon::fields().id().eq(parent_id)),
    );

    let n = filter.clone().count().exec(&mut db).await?;

    if n > 500 {
        return Err(ChildRegionError::TooManyChildren.into());
    }

    let all_regional_statuses = filter
        .include(RegionalTaxonStatus::fields().region())
        .exec(&mut db)
        .await?
        .into_iter()
        .fold(
            BTreeMap::new(),
            |mut accum: BTreeMap<u64, (Region, RegionHarvestWindowSummary)>, item| {
                accum
                    .entry(item.region_id)
                    .and_modify(|e| {
                        let e = &mut e.1;
                        e.harvest_window.start_doy =
                            [e.harvest_window.start_doy, item.harvest_window.start_doy]
                                .iter()
                                .flatten()
                                .copied()
                                .min();
                        e.harvest_window.end_doy =
                            [e.harvest_window.end_doy, item.harvest_window.end_doy]
                                .iter()
                                .flatten()
                                .copied()
                                .max();
                        if item.taxon_id == parent_id {
                            e.origin = item.origin;
                            e.conservation_status = item.conservation_status;
                        }
                    })
                    .or_insert((
                        item.region.into_inner(),
                        RegionHarvestWindowSummary {
                            harvest_window: item.harvest_window,
                            origin: (item.taxon_id == parent_id)
                                .then_some(item.origin)
                                .flatten(),
                            conservation_status: (item.taxon_id == parent_id)
                                .then_some(item.conservation_status)
                                .flatten(),
                        },
                    ));
                accum
            },
        );
    let mut regions = all_regional_statuses.into_values().collect::<Vec<_>>();
    regions.sort_by(|(a, _), (b, _)| {
        let mut res = a.category.cmp(&b.category).reverse();
        if res == Ordering::Equal {
            res = a.name.cmp(&b.name)
        }
        res
    });
    Ok(view! { taxon_regional_table(regions: &regions) })
}

#[memoize(as_ref)]
async fn detailed_taxon(cx: &Cx) -> topcoat::Result<Taxon> {
    let id = *taxon_id(cx)?;
    Ok(Taxon::filter_by_id(id)
        .include(Taxon::fields().vernaculars())
        .include(Taxon::fields().parent())
        .include(Taxon::fields().synonyms())
        .include(
            Taxon::fields()
                .children()
                .order_by(Taxon::fields().sequence().asc()),
        )
        .include(Taxon::fields().cleaning_procedures().citation_links())
        .include(Taxon::fields().propagation_procedures().propagation())
        .include(Taxon::fields().propagation_procedures().citation_links())
        .include(Taxon::fields().regional_statuses().region())
        .include(Taxon::fields().notes().citation_links())
        .include(Taxon::fields().photo())
        .include(Taxon::fields().ancestor_links().ancestor())
        .include(Taxon::fields().resources())
        .one()
        .exec(&mut db(cx))
        .await
        .ok_or_not_found()?)
}

#[component]
async fn propagation_actions(cx: &Cx, open_signal: Signal<bool>) -> topcoat::Result<impl View> {
    let actions = [Action {
        name: "Add Propagation Procedure".to_string(),
        permission: Some(PermissionCode::TaxonPropagationCreate),
        item_type: ActionItemType::Button,
        attrs: attributes! { @click=$(|_e: Event| open_signal.set(true)) },
    }];

    Ok(view! { action_menu(actions: &actions) })
}

#[component]
async fn note_actions(cx: &Cx) -> topcoat::Result<impl View> {
    let id = *taxon_id(cx)?;
    let actions = [Action {
        name: "Add Note".to_string(),
        permission: Some(PermissionCode::TaxonNoteCreate),
        item_type: ActionItemType::Link,
        attrs: attributes! { href=(href!(note::create, taxon_id: id)) },
    }];

    Ok(view! { action_menu(actions: &actions) })
}

#[component]
async fn menu_actions(cx: &Cx) -> topcoat::Result<impl View> {
    let id = *taxon_id(cx)?;
    let actions = [
        Action {
            name: "Sync Image".to_string(),
            permission: Some(PermissionCode::TaxonSync),
            item_type: ActionItemType::Button,
            attrs: attributes! {
                @click=$(async |_e: Event| {
                    sync_image(id).await;
                })
            },
        },
        Action {
            name: "Sync Regions".to_string(),
            permission: Some(PermissionCode::TaxonSync),
            item_type: ActionItemType::Button,
            attrs: attributes! {
                @click=$(async |_e: Event| {
                    sync_regions(id).await;
                })
            },
        },
    ];

    Ok(view! { action_menu(actions: &actions) })
}

#[page(
    GET "/taxa/{taxon_id}",
    with(Metadata::from_fn(
        async |cx| {
            let taxon = detailed_taxon(cx).await?;
            Ok(Metadata::new().subtitle(&taxon.complete_name))
        },
    ))
)]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let taxon = detailed_taxon(cx).await?;
    let id = taxon.id;
    let show_add_propagation = signal(cx, || false);

    Ok(view! {
        let ancestors = taxon
            .ancestor_links
            .get()
            .iter()
            .rev()
            .filter_map(|l| { if l.depth == 0 { None } else { Some(l.ancestor.get()) } })
            .collect::<Vec<_>>();
        ancestor_breadcrumbs(
            items: &ancestors,
            link_fn: |taxon| { href!(crate::taxa::details, taxon_id: taxon.id) },
            link_final: true
        )
        <h1 class="flex items-center">
            <span class="latin">(&taxon.complete_name)</span>
            badge(
                variant: BadgeVariant::Secondary,
                attrs: attributes! { class="mx-3" },
                (taxon.rank.to_string())
            )
            <div class="ms-auto">menu_actions()</div>
        </h1>
        <div class="flex flex-col gap-4">
            if let Some(photo) = taxon.photo.get() {
                if let Some(photo_url) = photo.medium_url.as_ref() {
                    <a
                        class="inline-block w-fit"
                        href=(photo
                            .original_url
                            .is_some()
                            .then(|| href!(default_photo, taxon_id: id)))
                    >
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
                                        <a href=(href!(details, taxon_id: child.id))>
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
                                        href=(href!(
                                            cleaning::details,
                                            taxon_id: procedure.taxon_id,
                                            cleaning::cleaning_id: procedure.id,
                                        ))
                                    >
                                        (&procedure.name)
                                    </a>
                                    for citation in procedure.citation_links.get() {
                                        <span class="align-super text-xs ps-1">
                                            "["
                                            <a
                                                href=(href!(
                                                    citation::details,
                                                    citation::citation_id: citation.citation_id,
                                                ))
                                            >
                                                (citation.citation_id)
                                            </a>
                                            "]"
                                        </span>
                                    }
                                </li>
                            }
                        </ul>
                    </div>
                </section>
            }
            if !taxon.propagation_procedures.get().is_empty() {
                <section>
                    <h2 class="flex items-center">
                        "Propagation Procedures"
                        <div class="ms-auto">
                            propagation_actions(
                                open_signal: show_add_propagation.clone()
                            )
                        </div>
                    </h2>
                    <div>
                        <ul>
                            for tp in taxon.propagation_procedures.get() {
                                <li>
                                    <a
                                        href=(href!(
                                            propagation::details,
                                            taxon_id: tp.taxon_id,
                                            propagation::propagation_id: tp.propagation_id,
                                        ))
                                    >
                                        (&tp.propagation.get().name)
                                    </a>
                                    for citation in tp.citation_links.get() {
                                        <span class="align-super text-xs ps-1">
                                            "["
                                            <a
                                                href=(href!(
                                                    citation::details,
                                                    citation::citation_id: citation.citation_id,
                                                ))
                                            >
                                                (citation.citation_id)
                                            </a>
                                            "]"
                                        </span>
                                    }
                                </li>
                            }
                        </ul>
                    </div>
                </section>
            }
            <section>
                <h2 class="flex items-center">
                    "Notes"
                    <div class="ms-auto">note_actions()</div>
                </h2>
                if !taxon.notes.get().is_empty() {
                    let notes: BTreeMap<TaxonNoteCategory, Vec<&TaxonNote>> = taxon
                        .notes
                        .get()
                        .iter()
                        .fold(
                            BTreeMap::new(),
                            |mut acc, member| {
                                acc.entry(member.category).or_default().push(member);
                                acc
                            },
                        );
                    <div class="flex flex-col gap-3">
                        for (category, notes) in notes {
                            <div>
                                <h4 class="mb-1">(category.to_string())</h4>
                                <ul>
                                    for note in notes {
                                        <li>
                                            <a
                                                href=(href!(note::details, taxon_id: taxon.id, note::note_id: note.id))
                                            >
                                                (&note.title)
                                            </a>
                                            for citation in note.citation_links.get() {
                                                <span class="align-super text-xs ps-1">
                                                    "["
                                                    <a
                                                        href=(href!(
                                                            citation::details,
                                                            citation::citation_id: citation.citation_id,
                                                        ))
                                                    >
                                                        (citation.citation_id)
                                                    </a>
                                                    "]"
                                                </span>
                                            }
                                        </li>
                                    }
                                </ul>
                            </div>
                        }
                    </div>
                }
            </section>

            <section>
                <h2>"Regions"</h2>
                <div>
                    error_boundary(
                        fallback: |e| {
                            let too_many_children = match e.downcast_ref::<
                                ChildRegionError,
                            >() {
                                Some(e) => matches!(e, ChildRegionError::TooManyChildren),
                                _ => false,
                            };

                            Ok(
                                view! {
                                    let regions = taxon
                                        .regional_statuses
                                        .get()
                                        .iter()
                                        .map(|rts| (rts.region.get().clone(), rts.into()))
                                        .collect::<Vec<(Region, RegionHarvestWindowSummary)>>();
                                    if too_many_children {
                                        <p class="text-muted-foreground">
                                            "Too many descendants. Select a child taxa to see regional status."
                                        </p>
                                    } else {
                                        // use the pre-fetched taxon-level data
                                        taxon_regional_table(regions: &regions)
                                    }
                                },
                            )
                        },
                        suspense(
                            fallback: view! { "Loading..." },
                            descendant_region_statuses(parent_id: taxon.id)
                        )
                    )
                </div>
            </section>
            if !taxon.synonyms.get().is_empty() {
                <section>
                    <h2>"Synonyms"</h2>
                    <div>
                        <ul>
                            for syn in taxon.synonyms.get() {
                                <li><span class="latin">(&syn.complete_name)</span></li>
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
                                    taxon.itis_id,
                                ))
                            >
                                "ITIS taxon info"
                            </a>
                        </li>
                        <li>
                            if let Some(id) = taxon.inaturalist_id {
                                <a href=(format!("https://www.inaturalist.org/taxa/{id}"))>
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
        if let Some(user) = current_user(cx).await {
            if user.has_permission(PermissionCode::TaxonPropagationCreate) {
                propagation::add_propagation_procedure_dialog(
                    form_attrs: attributes! {
                        method="POST"
                        action=(href!(propagation::add_propagation_procedure, taxon_id: taxon.id))
                    },
                    open_sig: show_add_propagation
                )
            }
        }
    })
}

#[procedure("/taxa/sync-image")]
pub async fn sync_image(cx: &Cx, taxon_id: u64) -> topcoat::Result<()> {
    require_user_with_permission(cx, PermissionCode::TaxonSync).await?;
    let mut db = db(cx);
    let taxon = Taxon::get_by_id(&mut db, taxon_id).await?;
    taxon.update_photo(&mut db).await?;
    Ok(())
}

#[procedure("/region/sync-regions")]
pub async fn sync_regions(cx: &Cx, taxon_id: u64) -> topcoat::Result<()> {
    require_user_with_permission(cx, PermissionCode::RegionSync).await?;
    let mut db = db(cx);

    // FIXME: allow min to be customized
    const MIN_SAMPLES: usize = 10;

    let regions = RegionalTaxonStatus::filter_by_taxon_id(taxon_id)
        .include(RegionalTaxonStatus::fields().taxon())
        .include(RegionalTaxonStatus::fields().region())
        .exec(&mut db)
        .await?;
    for rts in regions {
        match rts.query_harvest_info(&mut db).await {
            Ok(window) => {
                if window.n_samples.unwrap_or_default() < (MIN_SAMPLES).try_into().unwrap() {
                    warn!("Too few samples to calculate a harvest window");
                } else {
                    match RegionalTaxonStatus::update_by_id(rts.id)
                        .harvest_window(&window)
                        .exec(&mut db)
                        .await
                    {
                        Ok(_) => {
                            debug!(
                                "Updated harvest window for {} to {window}",
                                rts.taxon.get().reference()
                            )
                        }
                        Err(e) => {
                            warn!("Failed to update RegionalTaxonStatus: {e}")
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to calculate a harvest window for {}: {e}",
                    rts.taxon.get().reference()
                )
            }
        };
    }
    Ok(())
}
