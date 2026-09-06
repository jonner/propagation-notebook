use libpropagation::citation::Citation;
use topcoat::{
    router::href,
    view::{Attributes, class, component, view},
};

use crate::citation;

#[component]
pub async fn citation_link(citation: &Citation) -> topcoat::Result {
    view! {
        <a href=(href!(citation::details, citation::CitationId(citation.id)))>
            (citation.format_cse())
        </a>
    }
}

#[component]
pub async fn citation_list(
    citations: Vec<&Citation>,
    #[default] mut attrs: Attributes,
) -> topcoat::Result {
    view! {
        <ul
            class=(class!("text-sm flex flex-col gap-4", attrs.remove("class")))
            (attrs)
        >
            for citation in citations {
                <li>citation_link(citation: citation)</li>
            }
        </ul>
    }
}
