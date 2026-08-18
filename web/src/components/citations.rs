use libpropagation::citation::Citation;
use topcoat::{
    router::href,
    view::{component, view},
};

use crate::citation;

#[component]
pub async fn citation_list(citations: Vec<&Citation>) -> topcoat::Result {
    view! {
        <ul>
            for citation in citations {
                <li>
                    <a
                        href=(href!(citation::details, citation::CitationId(citation.id)))
                    >
                        (&citation.title)
                    </a>
                </li>
            }
        </ul>
    }
}
