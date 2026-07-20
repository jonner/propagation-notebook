use maud::{Markup, html};

pub mod propagation;
pub mod region;
pub mod taxonomy;

pub fn root() -> Markup {
    let title = "Propagation Notebook";
    html! {
        ( crate::templates::header(title) )
        h1 { (title) }
        ul {
            li { a href="/taxa/" { "Taxonomy" }}
            li { a href="/regions/" { "Regions" }}
            li { a href="/propagation/" { "Propagation Protocols" }}
        }
    }
}
