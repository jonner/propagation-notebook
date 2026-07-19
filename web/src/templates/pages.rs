use maud::{Markup, html};

pub fn root() -> Markup {
    let title = "Propagation Notebook";
    html! {
        ( crate::templates::header(title) )
        h1 { (title) }
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

pub mod propagation {
    use libpropagation::propagation::PropagationProcedure;
    use maud::{Markup, html};

    use crate::templates::header;

    pub fn root(procedures: &[PropagationProcedure]) -> Markup {
        let title = "Propagation Procedures";
        html! {
            (header(title))
            h1 { (title) }
            ul {
                @for p in procedures {
                    li {
                        a href={"./" (p.id)} { (p.name) }
                    }
                }
            }
        }
    }

    pub fn details(procedure: &PropagationProcedure) -> Markup {
        html! {
                (header(&procedure.name))
                h1 { (procedure.name) }
            dt { "ID" }
            dd { (procedure.id) }
            dt { "Name" }
            dd { (procedure.name) }
            dt { "Type" }
            dd { (procedure.r#type) }
            dt { "Notes" }
            dd { (procedure.notes.as_deref().unwrap_or_default()) }
            dt { "Instructions" }
            dd { (procedure.instructions) }
            dt { "Taxa"}
            dd {
                @if !procedure.taxa.get().is_empty() {
                    table {
                        tr {
                            th { "ID" }
                            th { "Name" }
                        }
                        @for tproc in procedure.taxa.get() {
                            tr {
                                td { (tproc.taxon.get().id) }
                                td { (tproc.taxon.get().complete_name) }

                            }
                        }
                    }
                } @else {
                    "None"
                }
            }


            dt { "Citations"}
            dd {
                @if !procedure.citations.get().is_empty() {
                    table {
                        tr {
                            th { "ID" }
                            th { "Name" }
                        }
                        @for citation in procedure.citations.get() {
                            tr {
                                td { (citation.id) }
                                td { (citation.title) }
                            }
                        }
                    }
                } @else {
                    "None"
                }
            }
        }
    }
}
