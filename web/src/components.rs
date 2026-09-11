use topcoat::{
    asset::{Asset, asset},
    icon::icon,
    view::{Attributes, Child, View, class, component, view},
};
use uuid::Uuid;

use crate::mdi;

pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod citations;
pub mod harvest;
pub mod input;
pub mod pagination;
pub mod pn;
pub mod tooltip;

const LEAFLET_INIT_SCRIPT: Asset = asset!("assets/leaflet-initialize.js");

#[component]
pub async fn leaflet_map(
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

#[component]
pub async fn taxa_grid(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div class=(class!("results-grid", attrs.remove("class"))) (attrs)>(child)</div>
    })
}
#[component]
pub async fn taxa_grid_item(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class!(
                "p-6 h-full flex flex-col items-center text-center justify-center",
                attrs.remove("class"),
            ))
            (attrs)
        >
            (child)
        </div>
    })
}

#[component]
pub async fn taxon_icon(
    url: Option<&String>,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    Ok(view! {
        if let Some(photo) = url.as_ref() {
            <img
                class=(class!("block rounded-xl border", attrs.remove("class")))
                (attrs)
                src=(photo)
            >
        } else {
            icon(data: mdi::LEAF_CIRCLE, size: 75, label: "Missing Image")
        }
    })
}
