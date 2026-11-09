use libpropagation::citation::Citation;
use topcoat::{
    context::Cx,
    router::href,
    view::{Attributes, View, class, component, view},
};

use crate::citation;

#[component]
pub async fn citation_link(cx: &Cx, citation: &Citation) -> topcoat::Result<impl View> {
    Ok(view! {
        <a href=(href!(citation::details, citation::citation_id: citation.id))>
            (citation.format_cse())
        </a>
    })
}

#[component]
pub async fn citation_list(
    cx: &Cx,
    citations: Vec<&Citation>,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <ul
            class=(class!("text-sm flex flex-col gap-4", attrs.remove("class")))
            (attrs)
        >
            for citation in citations {
                <li>citation_link(citation: citation)</li>
            }
        </ul>
    })
}
