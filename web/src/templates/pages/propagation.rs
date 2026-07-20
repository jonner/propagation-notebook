use libpropagation::propagation::PropagationProcedure;
use maud::{Markup, html};

use crate::templates::{Path, header};

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
    }
}
