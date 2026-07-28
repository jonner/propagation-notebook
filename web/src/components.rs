use libpropagation::citation::Citation;
use topcoat::view::{component, view};

use crate::util::{ModifyOffset, PageState};

#[component]
pub async fn citation_list(citations: Vec<&Citation>) -> topcoat::Result {
    view! {
        <ul>
            for citation in citations {
                <li>
                    <a href=(format!("/citation/{}", citation.id))>(&citation.title)</a>
                </li>
            }
        </ul>
    }
}

#[component]
pub async fn pagination_control<'p, T: ModifyOffset + Clone + Sync + Send + 'p>(
    state: &PageState,
    params: &'p T,
) -> topcoat::Result {
    view! {
        <nav class="mt-4">
            <ul class="flex gap-x-2">
                <li>
                    if let Some(offset) = state
                        .offset_for_page(state.current_page() - 1) {
                        <a
                            class="button"
                            href=(state
                                .query_with_offset(offset, params.clone()))
                        >
                            "< Prev"
                        </a>
                    } else {
                        "< Prev"
                    }
                </li>
                <li>
                    if let Some(offset) = state
                        .offset_for_page(state.current_page() + 1) {
                        <a
                            class="button"
                            href=(state
                                .query_with_offset(offset, params.clone()))
                        >
                            "Next >"
                        </a>
                    } else {
                        "Next >"
                    }
                </li>
            </ul>
        </nav>
    }
}
