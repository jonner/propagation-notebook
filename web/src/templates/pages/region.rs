use libpropagation::{
    region::{Region, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use maud::{Markup, html};
use tracing::trace;

use crate::{
    templates::{Path, header, map, page_control},
    util::PageState,
};

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

pub fn taxa_list(region: &Region, taxa: &[Taxon], page_state: &PageState) -> Markup {
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
        (page_control(page_state))
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
