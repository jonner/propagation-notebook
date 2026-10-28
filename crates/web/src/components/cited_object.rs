use libpropagation::citation::Citation;
use topcoat::{
    context::Cx,
    icon::icon,
    router::href,
    runtime::{Event, Signal, shard, signal},
    view::{Attributes, Child, View, attributes, class, component, element_id, view},
};

use crate::{
    assets::mdi,
    citation,
    components::{
        button::{ButtonVariant, button},
        card::{card, card_title},
        dialog::*,
        field::*,
        input::input,
        label::*,
        radio_group::*,
    },
    context::db,
};

#[component]
pub async fn cited_object_main_content(child: Child<'_>) -> topcoat::Result<impl View> {
    Ok(view! { card(attrs: attributes! { class="grow" }, (child)) })
}

#[component]
pub async fn cited_object_citation_section(child: Child<'_>) -> topcoat::Result<impl View> {
    Ok(view! {
        <section class="shrink min-w-0 md:w-md p-6">
            card_title("Citations")
            <ul class="ps-2 flex flex-col gap-2">(child)</ul>
        </section>
    })
}

#[component]
pub async fn cited_object_citation_item(child: Child<'_>) -> topcoat::Result<impl View> {
    Ok(view! { <li class="flex items-baseline gap-2">(child)</li> })
}

#[component]
pub async fn cited_object_layout(child: Child<'_>) -> topcoat::Result<impl View> {
    Ok(view! { <div class="flex gap-4 flex-wrap">(child)</div> })
}

#[shard]
async fn citation_options(cx: &Cx, query: String) -> topcoat::Result<impl View> {
    let n_citations = Citation::all().count().exec(&mut db(cx)).await?;
    let citations = if query.is_empty() {
        Vec::default()
    } else {
        Citation::filter(
            Citation::fields()
                .title()
                .like(format!("%{query}%"))
                .or(Citation::fields().author().like(format!("%{query}%")))
                .or(Citation::fields().url().like(format!("%{query}%")))
                .or(Citation::fields()
                    .container_title()
                    .like(format!("%{query}%"))),
        )
        .exec(&mut db(cx))
        .await?
    };
    Ok(view! {
        <div>(format!("{} / {n_citations} results shown", citations.len()))</div>
        if !citations.is_empty() {
            radio_group(
                for citation in citations {
                    let id = format!("citation-{}", citation.id);
                    <div class="flex items-center gap-2">
                        radio_group_item(
                            attrs: attributes! { id=(&id) name="citation_id" value=(citation.id) }
                        )
                        label(attrs: attributes! { for=(&id) }, (citation.format_cse()))
                    </div>
                }
            )
        }
        <a class="inline-flex items-center gap-2" href=(href!(citation::create))>
            icon(data: mdi::ADD)
            "Create a new citation source"
        </a>
    })
}

#[component]
async fn add_citation_form(cx: &Cx, #[default] attrs: Attributes) -> topcoat::Result<impl View> {
    let query = signal(cx, String::new);
    Ok(view! {
        <form (attrs)>
            field_set(
                field(
                    input(
                        attrs: attributes! {
                            type="text"
                            name="query"
                            value=(query.get())
                            placeholder="Search for an existing citation"
                            @input=$(|e: Event| query.set(e.target.value))
                        }
                    )
                )
                field(citation_options(query: query.get()))
            )
        </form>
    })
}

#[component]
pub async fn add_citation_dialog(
    cx: &Cx,
    open_sig: Signal<bool>,
    #[default] form_attrs: Attributes,
) -> topcoat::Result<impl View> {
    let add_citation_form_id = element_id(cx);
    Ok(view! {
        dialog(
            open: open_sig.get(),
            dialog_content(
                dialog_header(
                    dialog_title(
                        "Add a new citation for this taxon propagation procedure"
                    )
                )
                add_citation_form(
                    attrs: attributes! { id=(&add_citation_form_id) (form_attrs) }
                )
                dialog_footer(
                    button(
                        variant: ButtonVariant::Secondary,
                        attrs: attributes! {
                            @click=$(|e: topcoat::runtime::Event| {
                                e.prevent_default();
                                open_sig.set(false)
                            })
                        },
                        "Cancel"
                    )
                    button(
                        variant: ButtonVariant::Primary,
                        attrs: attributes! { form=(&add_citation_form_id) },
                        "Add"
                    )
                )
            )
        )
    })
}

#[component]
pub async fn remove_citation_dialog(
    remove_signal: Signal<Option<(u64, String)>>,
    #[default] form_attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        dialog(
            open: remove_signal.get().is_some(),
            dialog_content(
                if let Some((citation_id, citation_display)) = remove_signal.get() {
                    dialog_header(
                        dialog_title(
                            (format!("Remove citation {citation_id} from this object?"))
                        )
                        dialog_description("This action cannot be undone")
                    )
                    <section class="flex flex-col gap-3">
                        remove_citation_dialog_context_item(
                            remove_citation_dialog_context_item_term("Citation")
                            remove_citation_dialog_context_item_value(
                                (citation_display)
                            )
                        )
                        (child)
                    </section>
                    dialog_footer(
                        button(
                            variant: ButtonVariant::Secondary,
                            attrs: attributes! {
                                @click=$(|e: Event| {
                                    e.prevent_default();
                                    remove_signal.set(None)
                                })
                            },
                            "Cancel"
                        )
                        <form
                            @submit=$(|_e: Event| remove_signal.set(None))
                            (form_attrs)
                        >
                            button(variant: ButtonVariant::Destructive, "Delete")
                        </form>
                    )
                } else {
                    "Something went wrong"
                }
            )
        )
    })
}

#[component]
pub async fn remove_citation_dialog_context(
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div (attrs)>
            <h3>"Citation Context"</h3>
            <div class="flex flex-col gap-3">(child)</div>
        </div>
    })
}

#[component]
pub async fn remove_citation_dialog_context_item(
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! { <div (attrs)>(child)</div> })
}

#[component]
pub async fn remove_citation_dialog_context_item_term(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class!("text-muted-foreground text-sm", attrs.remove("class")))
            (attrs)
        >
            (child)
        </div>
    })
}
#[component]
pub async fn remove_citation_dialog_context_item_value(
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! { <div (attrs)>(child)</div> })
}
