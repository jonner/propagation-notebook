use topcoat::{
    icon::icon,
    view::{Attributes, View, class, component, view},
};

use crate::{leaflet::Map, mdi};

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

#[component]
pub async fn taxa_grid(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> topcoat::Result {
    view! {
        <div class=(class!("results-grid", attrs.remove("class"))) (attrs)>(child)</div>
    }
}
#[component]
pub async fn taxa_grid_item(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> topcoat::Result {
    view! {
        <div
            class=(class!(
                "p-6 h-full flex flex-col items-center text-center justify-center",
                attrs.remove("class"),
            ))
            (attrs)
        >
            (child)
        </div>
    }
}

#[component]
pub async fn taxon_icon(url: Option<&String>, #[default] mut attrs: Attributes) -> topcoat::Result {
    view! {
        if let Some(photo) = url.as_ref() {
            <img
                class=(class!("block rounded-xl border", attrs.remove("class")))
                (attrs)
                src=(photo)
            >
        } else {
            icon(data: mdi::LEAF_CIRCLE, size: 75, label: "Missing Image")
        }
    }
}
