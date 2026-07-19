use maud::{Markup, html};

pub fn root() -> Markup {
    html! {
        ( crate::templates::header("FIXME") )
        ul {
            li { a href="/taxonomy/" { "Taxonomy" }}
            li { a href="/regions/" { "Regions" }}
            li { a href="/propagation/" { "Propagation Protocols" }}
        }
    }
}

pub mod regions {
    use libpropagation::region::Region;
    use maud::{Markup, html};
    use tracing::trace;

    use crate::templates::{header, map};

    pub fn root(regions: &[Region]) -> Markup {
        trace!("rendering");
        let title = "Region List";
        html! {
            (header(title))
            h1 { (title) }
            ul {
                @for region in regions {
                    li { a href=(format!("./{}", region.id)) {(region.name)} }
                }
            }
        }
    }

    pub fn details(region: &Region) -> Markup {
        trace!("rendering");
        html! {
            (header(&region.name))
        h1 { (region.name) }
        dl {
            dt { "ID" }
            dd { (region.id) }
            dt { "Notes" }
            dd { (region.notes.as_deref().unwrap_or_default()) }
            dt { "Taxa" }
            dd { a href=(format!("./{}/taxa", region.id)) {
                (region.taxon_statuses.get().len())}
            }
            dt { "Geometry" }
            dd {
                @match region.geometry.as_ref() {
                    Some(value) => (map(value, None, None))
                    None => ""
                }
            }
        }
        }
    }
}
