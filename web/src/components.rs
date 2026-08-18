use jiff::ToSpan;
use libpropagation::propagation::PropagationProcedure;
use topcoat::{
    icon::icon,
    view::{Attributes, View, attributes, component, view},
};

use crate::{
    components::citations::citation_list,
    leaflet::Map,
    mdi,
    util::{ModifyOffset, PageState},
};

pub mod badge;
pub mod citations;
pub mod harvest;
pub mod tooltip;

enum PageLinkType {
    Ellipsis,
    Page(usize),
    Icon(usize, View),
}

#[component]
pub async fn pagination_control<'p, T: ModifyOffset + Clone + Sync + Send + 'p>(
    state: &PageState,
    params: &'p T,
    #[default(2)] context: usize,
) -> topcoat::Result {
    let mut links = Vec::with_capacity(context * 2 + 5);
    let cur = state.current_page();
    let first = cur.saturating_sub(context).max(1);
    let last = (cur + context).min(state.total_pages());
    if cur > 1 {
        links.push(PageLinkType::Icon(
            cur - 1,
            view! {
                icon(
                    data: mdi::NAVIGATE_BEFORE,
                    label: "Previous",
                    attrs: attributes! { class="icon" }
                )
            }
            .unwrap(),
        ));
    }
    if first > 1 {
        links.push(PageLinkType::Page(1));
    }
    if first > 2 {
        links.push(PageLinkType::Ellipsis)
    }
    for n in first..=last {
        links.push(PageLinkType::Page(n));
    }
    if last < (state.total_pages() - 1) {
        links.push(PageLinkType::Ellipsis)
    }
    if last < state.total_pages() {
        links.push(PageLinkType::Page(state.total_pages()));
    }
    if cur < last {
        links.push(PageLinkType::Icon(
            cur + 1,
            view! {
                icon(
                    data: mdi::NAVIGATE_NEXT,
                    label: "Next",
                    attrs: attributes! { class="icon" }
                )
            }
            .unwrap(),
        ));
    }
    view! {
        <nav class="flex gap-3 items-center my-4">
            <ul class="contents">
                for item in links {
                    <li>
                        match item {
                            PageLinkType::Ellipsis => {
                                icon(
                                    data: mdi::ELLIPSIS_HORIZONTAL,
                                    label: "skipped",
                                    attrs: attributes! { class="icon" }
                                )
                            }
                            PageLinkType::Page(n) => {
                                if n == state.current_page() {
                                    <span class="inline-block font-bold self-center">
                                        (n.to_string())
                                    </span>
                                } else {
                                    <a
                                        href=(state
                                            .query_with_offset(
                                                state.offset_for_page(n).unwrap_or_default(),
                                                params.clone(),
                                            ))
                                    >
                                        (n.to_string())
                                    </a>
                                }
                            }
                            PageLinkType::Icon(n, view) => {
                                <a
                                    class="button"
                                    href=(state
                                        .query_with_offset(
                                            state.offset_for_page(n).unwrap_or_default(),
                                            params.clone(),
                                        ))
                                >
                                    (view)
                                </a>
                            }
                        }
                    </li>
                }
            </ul>
        </nav>
    }
}

#[component]
pub async fn propagation_details(procedure: &PropagationProcedure) -> topcoat::Result {
    view! {
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
    }
}

pub struct Breadcrumb {
    pub url: Option<String>,
    pub text: String,
}

#[component]
pub async fn breadcrumbs(
    items: Vec<Breadcrumb>,
    #[default] ellipsize: Option<usize>,
) -> topcoat::Result {
    let do_ellipsize = ellipsize.map(|n| items.len() > n + 1) == Some(true);
    let mut breadcrumb_iter = items.into_iter();
    let Some(root) = breadcrumb_iter.next() else {
        return view! {};
    };
    let rest: Vec<_> = if let Some(ellipsize) = ellipsize {
        breadcrumb_iter.rev().take(ellipsize).rev().collect()
    } else {
        breadcrumb_iter.collect()
    };
    view! {
        <nav class="breadcrumbs">
            <ol>
                <li><a href=(root.url)>(&root.text)</a></li>
                if do_ellipsize {
                    <li>
                        icon(
                            data: mdi::NAVIGATE_NEXT,
                            label: "separator",
                            attrs: attributes! { class="icon" }
                        )
                        <a>
                            icon(
                                data: mdi::ELLIPSIS_HORIZONTAL,
                                label: "skipped",
                                attrs: attributes! { class="icon" }
                            )
                        </a>
                    </li>
                }
                for item in rest {
                    <li>
                        icon(
                            data: mdi::NAVIGATE_NEXT,
                            label: "separator",
                            attrs: attributes! { class="icon" }
                        )
                        <a href=(item.url)>(&item.text)</a>
                    </li>
                }
            </ol>
        </nav>
    }
}

#[component]
pub async fn leaflet_map(
    geometry: &geojson::Geometry,
    #[default] attrs: Attributes,
) -> topcoat::Result {
    let leaflet_script = Map::new(geometry);
    view! {
        <div id=(&leaflet_script.id) (attrs)></div>
        (leaflet_script)
    }
}

#[component]
pub async fn week_navigator(date: jiff::civil::Date) -> topcoat::Result {
    let prev_date = date - 7.days();
    let prev_link = format!("?date={}", prev_date);
    let next_date = date + 7.days();
    let next_link = format!("?date={}", next_date);
    let date_str = date.strftime("%B %d").to_string();
    view! {
        <nav class="flex gap-4">
            <ol class="contents">
                <li><a href=(prev_link)>"Prev"</a></li>
                <li>(date_str)</li>
                <li><a href=(next_link)>"Next"</a></li>
            </ol>
        </nav>
    }
}
