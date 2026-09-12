use libpropagation::taxonomy::Taxon;
use topcoat::{
    asset::{Asset, asset},
    context::Cx,
    icon::icon,
    router::href,
    runtime::{Event, shard, signal},
    view::{Attributes, Child, View, ViewExt, attributes, class, component, view},
};
use uuid::Uuid;

use crate::{
    mdi,
    taxa::{self, TaxonId},
    util::db,
};

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

#[shard]
pub async fn taxon_search_results(cx: &Cx, query_string: String) -> topcoat::Result<impl View> {
    let nothing = view! {}.boxed();
    if query_string.len() < 2 {
        return Ok(nothing);
    }
    let mut db = db(cx);
    const LIMIT: u64 = 20;
    let query = Taxon::filter(Taxon::search_filter(&query_string))
        .include(Taxon::fields().photo())
        .order_by((
            Taxon::fields().sequence().asc(),
            Taxon::fields().complete_name().asc(),
        ));
    let total = query.clone().count().exec(&mut db).await?;
    let taxa = query.limit(LIMIT as usize).exec(&mut db).await?;
    if taxa.is_empty() {
        return Ok(nothing);
    }
    Ok(view! {
        <div
            class="flex flex-col absolute left-0 right-0 max-h-lg gap-2 rounded-xl border border-border p-3 text-sm text-foreground shadow-sm bg-background/80 z-50"
        >
            <ul class="contents">
                for taxon in taxa.iter() {
                    <li class="py-1">
                        <span class="latin">
                            <a href=(href!(taxa::details, TaxonId(taxon.id)))>
                                (&taxon.complete_name)
                            </a>
                        </span>
                    </li>
                }
                if total > LIMIT {
                    <li>(format!("...and {} more", total - LIMIT))</li>
                }
            </ul>
        </div>
    }
    .boxed())
}

#[component]
pub async fn taxon_search_bar(
    cx: &Cx,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    let query_string = signal(cx, String::new);
    Ok(view! {
        <div class=(class!("relative", attrs.remove("class"))) (attrs)>
            <form method="get" action=(href!(taxa::search)) class="contents">
                input::input(
                    attrs: attributes! {
                        type="text"
                        name="q"
                        placeholder="Search for a taxon"
                        autocomplete="off"
                        @input=$(|e: Event| query_string.set(e.target.value))
                        class="text-foreground hover:opacity-80 focus-within:opacity-80 opacity-50"
                    }
                )
                taxon_search_results(query_string: $(query_string.get()))
            </form>
        </div>
    })
}
