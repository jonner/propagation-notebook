use topcoat::{
    icon::icon,
    view::{Attributes, attributes, component, view},
};

use crate::{leaflet::Map, mdi};

pub mod badge;
pub mod button;
pub mod card;
pub mod citations;
pub mod harvest;
pub mod input;
pub mod pagination;
pub mod tooltip;

pub struct Breadcrumb {
    pub url: Option<String>,
    pub text: String,
}

#[component]
pub async fn breadcrumbs(
    items: Vec<Breadcrumb>,
    #[default] ellipsize: Option<usize>,
) -> topcoat::Result {
    let do_ellipsize = ellipsize.map(|n| items.len() > n + 1) == Some(true);
    let mut breadcrumb_iter = items.into_iter();
    let Some(root) = breadcrumb_iter.next() else {
        return view! {};
    };
    let rest: Vec<_> = if let Some(ellipsize) = ellipsize {
        breadcrumb_iter.rev().take(ellipsize).rev().collect()
    } else {
        breadcrumb_iter.collect()
    };
    view! {
        <nav class="breadcrumbs">
            <ol>
                <li><a href=(root.url)>(&root.text)</a></li>
                if do_ellipsize {
                    <li>
                        icon(
                            data: mdi::NAVIGATE_NEXT,
                            label: "separator",
                            attrs: attributes! { class="icon" }
                        )
                        <a>
                            icon(
                                data: mdi::ELLIPSIS_HORIZONTAL,
                                label: "skipped",
                                attrs: attributes! { class="icon" }
                            )
                        </a>
                    </li>
                }
                for item in rest {
                    <li>
                        icon(
                            data: mdi::NAVIGATE_NEXT,
                            label: "separator",
                            attrs: attributes! { class="icon" }
                        )
                        <a href=(item.url)>(&item.text)</a>
                    </li>
                }
            </ol>
        </nav>
    }
}

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
