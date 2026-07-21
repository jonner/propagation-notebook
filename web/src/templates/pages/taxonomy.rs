use libpropagation::{
    collecting::CleaningProcedure,
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use maud::{Markup, html};
use tracing::trace;

use crate::{
    taxonomy::TaxaListParams,
    templates::{Path, layout, pagination_control},
    util::PageState,
};

pub fn root(taxa: &[Taxon], page_state: &PageState, params: &TaxaListParams) -> Markup {
    let content = html! {
        ul {
            @for taxon in taxa.iter() {
                li { a href=(taxon.path()) {(taxon.complete_name)} }
            }
        }
        (pagination_control(page_state, params))
    };

    layout("Taxon List", content)
}

pub fn details(taxon: &Taxon) -> Markup {
    trace!("rendering");
    let content = html! {
        dt { "ID" }
        dd { (taxon.id) }
        dt { "Name" }
        dd { (taxon.complete_name) }
        dt { "Rank" }
        dd { (taxon.rank) }

        dt { "Parent" }
        dd {
            @match taxon.parent.get() {
                Some(p) => a href=(p.path()) { (p.complete_name) },
                None => "",
            }
        }

        dt { "Synonyms" }
        dd {
            ul {
                @for syn in taxon.synonyms.get() {
                    li { (syn.complete_name) }
                }
            }
        }

        dt { "Common Name(s)" }
        dd {
            ul {
                @for cn in taxon.vernaculars.get(){
                    li { (cn.name) }
                }
            }
        }

        dt { "Child taxa" }
        dd {
            ul {
                @for child in taxon.children.get() {
                    li { a href=(child.path()) { (child.complete_name) } }
                }
            }
        }

        dt { "ITIS taxon ID" }
        dd { (taxon.itis_id) }

        dt { "iNaturalist taxon ID" }
        dd { (taxon.inaturalist_id.map(|v| v.to_string()).unwrap_or_default()) }
        @if let Some(collecting_data) = &taxon.collecting_data.get() {
            dt { "Ripening" }
            dd { (collecting_data.ripening_indicators.as_deref().unwrap_or_default()) }

            dt { "Harvesting Notes" }
            dd { (collecting_data.harvesting_notes.as_deref().unwrap_or_default()) }

            dt { "Storage Conditions" }
            dd { (collecting_data.storage.as_deref().unwrap_or_default()) }

            dt { "Storage Life" }
            dd { (collecting_data.storage_life.as_deref().unwrap_or_default()) }
        }
        dt { "Seed Cleaning" }
        dd {
            @if !taxon.cleaning_procedures.get().is_empty() {
                table {
                    tr {
                        th { "ID" }
                        th { "Name" }
                    }
                    @for procedure in taxon.cleaning_procedures.get() {
                        tr {
                            td { (procedure.id) }
                            td { a href=(procedure.path()) { (procedure.name) } }
                        }
                    }
                }

            }
        }
        dt { "Propagation Procedures" }
        dd {
            @if !taxon.propagation_procedures.get().is_empty() {
                table {
                    tr {
                        th { "ID" }
                        th { "Name" }
                    }
                    @for tp in taxon.propagation_procedures.get() {
                        tr {
                            td { (tp.propagation.get().id) }
                            td { a href=(tp.path()) { (tp.propagation.get().name) } }
                        }
                    }
                }

            }
        }
        dt { "Regions" }
        dd {
            @if !taxon.regional_statuses.get().is_empty() {
                table {
                    tr {
                        th { "ID" }
                        th { "Name" }
                        th { "Origin" }
                        th { "Harvest Window" }
                    }
                    @for rs in taxon.regional_statuses.get() {
                        tr {
                            td { (rs.region.get().id) }
                            td { a href=(rs.path()) { (rs.region.get().name) } }
                            td { (rs.origin.map(|v| v.to_string()).unwrap_or_default()) }
                            td { (rs.harvest_window) }
                        }
                    }
                }
            }
        }
        dt { "Notes" }
        dd {
            @if !taxon.notes.get().is_empty() {
                table {
                    tr {
                        th { "ID" }
                        th { "Name" }
                    }
                    @for note in taxon.notes.get() {
                        tr {
                            td { (note.id) }
                            td { (note.text) }
                        }
                    }
                }

            }
        }
    };

    layout(&taxon.complete_name, content)
}

pub fn propagation_details(tp: &TaxonPropagationProcedure) -> Markup {
    let proc = tp.propagation.get();
    let taxon = tp.taxon.get();
    let title = format!("{} for {}", proc.name, taxon.complete_name);
    let content = html! {
        dt { "Procedure" }
        dd { a href=(tp.propagation.get().path()) { (tp.propagation.get().name) } }
        dt { "Taxon" }
        dd { a href=(taxon.path()) { (taxon.complete_name) } }
        dt { "Confidence" }
        dd { (tp.confidence.map(|v| v.to_string()).unwrap_or_default()) }
        dt { "Taxon-specific notes" }
        dd { (tp.notes.as_deref().unwrap_or_default()) }
        dt { "Citations" }
        dd {
            table {
                tr {
                    th { "ID" }
                    th { "Name" }
                }
                @for cl in tp.citation_links.get() {
                    tr {

                        td { (cl.citation.get().id) }
                        td { (cl.citation.get().title) }
                    }
                }
            }
        }
    };
    layout(&title, content)
}

pub fn cleaning_details(proc: &CleaningProcedure) -> Markup {
    let taxon = proc.taxon.get();
    let title = format!("{} for {}", proc.name, taxon.complete_name);
    let content = html! {
        dt { "Taxon" }
        dd { a href=(taxon.path()) { (taxon.complete_name) } }
        dt { "Instructions" }
        dd { (proc.instructions) }
        dt { "Additional Notes" }
        dd { (proc.notes.as_deref().unwrap_or_default()) }
        dt { "Citations" }
        dd {
            table {
                tr {
                    th { "ID" }
                    th { "Name" }
                }
                @for cl in proc.citation_links.get() {
                    tr {

                        td { (cl.citation.get().id) }
                        td { (cl.citation.get().title) }
                    }
                }
            }
        }
    };
    layout(&title, content)
}
