use libpropagation::taxonomy::Taxon;
use topcoat::{
    context::Cx,
    router::href,
    runtime::{Event, shard, signal},
    view::{Attributes, View, ViewExt, attributes, class, component, view},
};

use crate::{
    components::{button::button, input::input},
    context::db,
    taxa,
};

use super::floating::{
    FLOATING_SCOPE, FloatingAlign, FloatingAnchor, FloatingSide, floating_variants,
};

#[shard("/taxa/search/results")]
pub async fn taxon_search_results(
    cx: &Cx,
    query_string: String,
    visible: bool,
) -> topcoat::Result<impl View> {
    let nothing = view! {}.boxed();
    if !visible || query_string.len() < 2 {
        return Ok(nothing);
    }
    let mut db = db(cx);
    const LIMIT: u64 = 40;
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
            class=(class!(
                floating_variants(
                    FloatingAnchor::Scope,
                    FloatingSide::Bottom,
                    FloatingAlign::Start,
                ),
                "flex flex-col gap-2 p-3",
                "max-h-100 overflow-y-auto",
                "text-sm text-foreground bg-background/95 z-50",
                "rounded-xl border border-border shadow-sm",
                "[width:max(anchor-size(width),18rem)]",
            ))
        >
            <ul class="contents">
                for taxon in taxa.iter() {
                    <li class="py-1">
                        <a
                            class="block latin"
                            href=(href!(taxa::details, taxa::taxon_id: taxon.id))
                        >
                            (&taxon.complete_name)
                        </a>
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

/// A taxon search field with live results.
///
/// Set `show_button` to include a submit button next to the field. The header
/// uses the field on its own; the landing-page hero adds the button.
#[component]
pub async fn taxon_search_bar(
    cx: &Cx,
    #[default] show_button: bool,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    let query_string = signal(cx, String::new);
    let search_focus = signal(cx, || false);
    Ok(view! {
        <div
            @focusin=$(|_e: Event| search_focus.set(true))
            @focusout=$(|_e: Event| search_focus.set(false))
            class=(class!("relative", FLOATING_SCOPE, attrs.remove("class")))
            (attrs)
        >
            <form method="get" action=(href!(taxa::search)) class="contents">
                <div class="flex gap-2">
                    input(
                        attrs: attributes! {
                            type="text"
                            name="q"
                            placeholder="Search for a taxon"
                            autocomplete="off"
                            class="grow"
                            @input=$(|e: Event| query_string.set(e.target.value))
                        }
                    )
                    if show_button {
                        button(attrs: attributes! { type="submit" }, "Search")
                    }
                </div>
                taxon_search_results(
                    query_string: $(query_string.get()),
                    visible: $(search_focus.get())
                )
            </form>
        </div>
    })
}
