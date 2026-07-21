use libpropagation::propagation::PropagationProcedure;
use maud::{Markup, html};

use crate::templates::{Path, layout};

pub fn root(procedures: &[PropagationProcedure]) -> Markup {
    let title = "Propagation Procedures";
    let content = html! {
        ul {
            @for p in procedures {
                li {
                    a href={"./" (p.id)} { (p.name) }
                }
            }
        }
    };
    layout(title, content)
}

pub fn details(procedure: &PropagationProcedure) -> Markup {
    let content = html! {
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
                            td { a href=(tproc.taxon.get().path()) { (tproc.taxon.get().complete_name) } }

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
    };
    layout(&procedure.name, content)
}
