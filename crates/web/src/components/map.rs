use topcoat::{
    asset::{Asset, asset},
    context::Cx,
    view::{Attributes, View, component, view},
};
use uuid::Uuid;

const LEAFLET_INIT_SCRIPT: Asset = asset!("assets/leaflet-initialize.js");

#[component]
pub async fn leaflet_map(
    cx: &Cx,
    geometry: &geojson::Geometry,
    #[default] attrs: Attributes,
) -> topcoat::Result<impl View> {
    let id = Uuid::new_v4().to_string();
    Ok(view! {
        <div id=(&id) (attrs)></div>
        <script src=(LEAFLET_INIT_SCRIPT)></script>
        <script>
            (format!("var geojson = {};", geometry))
            (format!("var id = '{}';", id))
            "initializeLeaflet(id, geojson);"
        </script>
    })
}
