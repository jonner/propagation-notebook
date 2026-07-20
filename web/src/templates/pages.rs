use maud::{Markup, html};

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

pub mod regions {
    use libpropagation::{
        region::{Region, RegionalTaxonStatus},
        taxonomy::Taxon,
    };
    use maud::{Markup, html};
    use tracing::trace;

    use crate::templates::{Path, header, map};

    pub fn root(regions: &[Region]) -> Markup {
        trace!("rendering");
        let title = "Region List";
        html! {
            (header(title))
            h1 { (title) }
            ul {
                @for region in regions {
                    li { a href=(region.path()) { (region.name) } }
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

    pub fn taxa_list(region: &Region, taxa: &[Taxon]) -> Markup {
        html! {
            (header(&region.name))
            h1 { (region.name) }
            ul {
                @for taxon in taxa {
                    @for rts in taxon.regional_statuses.get() {
                        @if rts.region_id == region.id {
                            li { a href=(rts.path()) { (taxon.complete_name) }}
                        }
                    }
                }
            }
        }
    }

    pub fn taxon_details(status: &RegionalTaxonStatus) -> Markup {
        let region = status.region.get();
        let taxon = status.taxon.get();
        let title = format!("{} in {}", taxon.complete_name, region.name);

        html! {
            (header(&title))
            h1 { (title) }
            dt { "Taxon" }
            dd {  a href=(taxon.path()) { (taxon.complete_name) } }
            dt { "Region" }
            dd {  a href=(region.path()) { (region.name) } }
            dt { "Origin" }
            dd { (status.origin.map(|v| v.to_string()).unwrap_or_default() )}
            dt { "C-value" }
            dd { (status.c_value.map(|v| v.to_string()).unwrap_or_default()) }
            dt { "Conservation Status" }
            dd { (status.conservation_status.map(|v| v.to_string()).unwrap_or_default() )}
            dt { "Wetland Indicator" }
            dd { (status.wetland_indicator.map(|v| v.to_string()).unwrap_or_default() )}
            dt { "Harvest Window" }
            dd { (status.harvest_window.to_string() )}
        }
    }
}

pub mod propagation {
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
}

pub mod taxonomy {
    use libpropagation::taxonomy::Taxon;
    use maud::{Markup, html};
    use tracing::trace;

    use crate::templates::{Path, header};

    pub fn root(taxa: &[Taxon]) -> Markup {
        trace!("rendering");
        let title = "Taxon List";
        html! {
            (header(title))
            h1 { (title) }
            ul {
                @for taxon in taxa {
                    li { a href=(taxon.path()) {(taxon.complete_name)} }
                }
            }
        }
    }

    pub fn details(taxon: &Taxon) -> Markup {
        trace!("rendering");
        html! {
            (header(&taxon.complete_name))
            h1 { (taxon.complete_name) }
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
                                td { (procedure.name) }
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
                                td { a href=(tp.propagation.get().path()) { (tp.propagation.get().name) } }
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
                                td { a href=(rs.region.get().path()) { (rs.region.get().name) } }
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
        }
    }
}
