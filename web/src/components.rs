use topcoat::view::{Attributes, component, view};

use crate::leaflet::Map;

pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod citations;
pub mod harvest;
pub mod input;
pub mod pagination;
pub mod tooltip;

#[component]
pub async fn leaflet_map(
    geometry: &geojson::Geometry,
    #[default] attrs: Attributes,
) -> topcoat::Result {
    let leaflet_script = Map::new(geometry);
    view! {
        <div id=(&leaflet_script.id) (attrs)></div>
        (leaflet_script)
    }
}
