use libpropagation::{citation::Citation, propagation::PropagationProcedure};
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
