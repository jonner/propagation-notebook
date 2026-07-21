use libpropagation::{
    region::{Region, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use maud::{Markup, html};
use tracing::trace;

use crate::{
    templates::{Path, layout, map, pagination_control},
    util::{PageQueryParams, PageState},
};

pub fn region_list(regions: &[Region]) -> Markup {
    trace!("rendering");
    let title = "Region List";
    let content = html! {
        ul {
            @for region in regions {
                li { a href=(region.path()) { (region.name) } }
            }
        }
    };
    layout(title, content)
}

pub fn region_details(region: &Region) -> Markup {
    trace!("rendering");
    let content = html! {
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
    };
    layout(&region.name, content)
}

pub fn region_taxa_list(
    region: &Region,
    taxa: &[Taxon],
    page_state: &PageState,
    params: &PageQueryParams,
) -> Markup {
    let content = html! {
        ul {
            @for taxon in taxa {
                @for rts in taxon.regional_statuses.get() {
                    @if rts.region_id == region.id {
                        li { a href=(rts.path()) { (taxon.complete_name) }}
                    }
                }
            }
        }
        (pagination_control(page_state, params))
    };
    layout(&region.name, content)
}

pub fn region_taxon_status(status: &RegionalTaxonStatus) -> Markup {
    let region = status.region.get();
    let taxon = status.taxon.get();
    let title = format!("{} in {}", taxon.complete_name, region.name);

    let content = html! {
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
    };
    layout(&title, content)
}
