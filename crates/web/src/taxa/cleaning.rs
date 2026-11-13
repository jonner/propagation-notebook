use libpropagation::{
    auth::PermissionCode, citation::CleaningProcedureCitation, cleaning::CleaningProcedure,
};
use serde::Deserialize;
use topcoat::{
    context::{Cx, memoize},
    icon::icon,
    metadata::Metadata,
    router::{
        content::Form,
        error::{RouterErrorExt, SeeOther, see_other},
        href, page, path_param, route,
    },
    runtime::{Event, signal},
    view::{Attributes, View, attributes, component, view},
};
use tracing::trace;

use crate::{
    Subtitle,
    actions::{Action, ActionItemType, permitted_actions},
    assets::mdi,
    components::{
        action_menu::*,
        breadcrumb::*,
        card::*,
        citations::citation_list,
        cited_object::*,
        field::*,
        input::*,
        pn::{item_term, item_value},
        textarea::*,
    },
    context::{current_user, db, require_user_with_permission},
    taxa,
};

path_param!(pub cleaning_id: u64 else bad_request);
path_param!(pub citation_id: u64 else bad_request);

#[memoize(as_ref)]
async fn cleaning_procedure(cx: &Cx) -> topcoat::Result<CleaningProcedure> {
    let taxon_id = *taxa::taxon_id(cx)?;
    let cleaning_id = *cleaning_id(cx)?;
    Ok(CleaningProcedure::filter(
        CleaningProcedure::fields()
            .taxon_id()
            .eq(taxon_id)
            .and(CleaningProcedure::fields().id().eq(cleaning_id)),
    )
    .include(CleaningProcedure::fields().taxon())
    .include(CleaningProcedure::fields().citation_links().citation())
    .one()
    .exec(&mut db(cx))
    .await
    .ok_or_not_found()?)
}

#[page(
    "/taxa/{taxon_id}/cleaning/{cleaning_id}",
    with(Metadata::from_fn(
        async |cx| {
            let proc = cleaning_procedure(cx).await?;
            Ok(
                Metadata::new().subtitle(
                    format!(
                        "Cleaning procedure {} for {}",
                        proc.id,
                        proc.taxon.get().complete_name,
                    ),
                ),
            )
        },
    ))
)]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let user = current_user(cx).await;
    let delete_dialog_open = signal(cx, || false);

    let proc = cleaning_procedure(cx).await?;
    trace!(?proc);
    let actions = [
        Action {
            name: "Edit".to_string(),
            permission: Some(PermissionCode::TaxonCleaningEdit),
            attrs: attributes! {
                href=(href!(modify, taxa::taxon_id: proc.taxon_id, cleaning_id: proc.id))
            },
            item_type: ActionItemType::Link,
        },
        Action {
            name: "Delete".to_string(),
            permission: Some(PermissionCode::TaxonCleaningDelete),
            attrs: attributes! {
                @click=$(|_e: topcoat::runtime::Event| {
                    delete_dialog_open.set(true);
                })
                class="text-destructive"
            },
            item_type: ActionItemType::Button,
        },
    ];

    Ok(view! {
        let taxon = proc.taxon.get();
        <div class="flex flex-col gap-4">
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(taxa::details, taxa::taxon_id: taxon.id)) },
                                (&taxon.complete_name)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_page((format!("Cleaning Procedure {}", proc.id)))
                        )
                    )
                )
                <h1 class="flex items-baseline">
                    <span>
                        <span>(&proc.name)</span>
                        " for "
                        <span class="latin">(&taxon.complete_name)</span>
                    </span>
                    let permitted_actions = permitted_actions(user, &actions);
                    if !permitted_actions.is_empty() {
                        <div class="ms-auto">
                            action_menu(actions: permitted_actions.as_slice())
                        </div>
                    }
                </h1>
            </hgroup>
            <section>
                <h2>"Instructions"</h2>
                <div class="text-2xl">(&proc.instructions)</div>
            </section>
            if let Some(notes) = proc.notes.as_ref() {
                <section>
                    <h2>"Additional Notes"</h2>
                    <div>(notes)</div>
                </section>
            }
            <section>
                <h2>"Citations"</h2>
                <div>
                    citation_list(
                        citations: proc
                            .citation_links
                            .get()
                            .iter()
                            .map(|cl| cl.citation.get())
                            .collect()
                    )
                </div>
            </section>
        </div>
    })
}

#[component]
async fn cleaning_procedure_form(
    cx: &Cx,
    #[default] attrs: Attributes,
    procedure: &CleaningProcedure,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <form (attrs)>
            field_set(
                field(
                    field_label("Name")
                    input(attrs: attributes! { name="name" value=(&procedure.name) })
                )
                field(
                    field_label("Instructions")
                    textarea(
                        attrs: attributes! { rows="8" name="instructions" },
                        (&procedure.instructions)
                    )
                )
                field(
                    field_label("Additional Notes")
                    textarea(
                        attrs: attributes! { rows="5" name="notes" },
                        (procedure.notes.as_deref().unwrap_or_default())
                    )
                )
            )
        </form>
    })
}

#[page(
    "/taxa/{taxon_id}/cleaning/{cleaning_id}/edit",
    with(Metadata::from_fn(
        async |cx| {
            let proc = cleaning_procedure(cx).await?;
            Ok(
                Metadata::new().subtitle(
                    format!(
                        "Edit cleaning procedure {} for {}",
                        proc.id,
                        proc.taxon.get().complete_name,
                    ),
                ),
            )
        },
    ))
)]
pub async fn modify(cx: &Cx) -> topcoat::Result<impl View> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonCleaningEdit).await?;
    let cleaning_procedure = cleaning_procedure(cx).await?;

    let delete_citation_data = signal(cx, || None::<(u64, String)>);
    let add_citation_dialog_open = signal(cx, || false);

    Ok(view! {
        let taxon = cleaning_procedure.taxon.get();
        <section>
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(taxa::details, taxa::taxon_id: taxon.id)) },
                                (&taxon.complete_name)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page("Cleaning procedures"))
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! {
                                    href=(href!(
                                        details,
                                        taxa::taxon_id: taxon.id,
                                        cleaning_id: cleaning_procedure.id,
                                    ))
                                },
                                (cleaning_procedure.id)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page("Edit"))
                    )
                )
                <h1>"Edit cleaning procedure"</h1>
            </hgroup>
        </section>
        cited_object_layout(
            cited_object_main_content(
                card_header(card_title("Procedure Details"))
                card_content(cleaning_procedure_form(procedure: &cleaning_procedure))
            )
            cited_object_citation_section(
                for citation_link in cleaning_procedure.citation_links.get() {
                    let citation = citation_link.citation.get();
                    cited_object_citation_item(
                        let citation_data = (citation.id, citation.format_cse());
                        <a
                            class="text-destructive"
                            href="#"
                            @click=$(|e: Event| {
                                e.prevent_default();
                                delete_citation_data.set(Some(citation_data));
                            })
                        >
                            icon(data: mdi::DELETE)
                        </a>
                        <span class="text-muted-foreground">
                            (citation.id)
                            "."
                        </span>
                        <span class="text-wrap">(citation.format_cse())</span>
                    )
                }
                cited_object_citation_item(
                    <a
                        class="flex items-baseline gap-2"
                        href="#"
                        @click=$(|e: Event| {
                            e.prevent_default();
                            add_citation_dialog_open.set(true);
                        })
                    >
                        icon(data: mdi::ADD)
                        "Add citation"
                    </a>
                )
            )
        )
        let citation_ids = cleaning_procedure
            .citation_links
            .get()
            .iter()
            .map(|link| link.citation_id)
            .collect::<Vec<_>>();
        add_citation_dialog(
            open_sig: add_citation_dialog_open.clone(),
            exclude: citation_ids,
            form_attrs: attributes! {
                method="POST"
                action=(href!(
                    add_citation,
                    taxa::taxon_id: taxon.id,
                    cleaning_id: cleaning_procedure.id,
                ))
            }
        )
        remove_citation_dialog(
            remove_signal: delete_citation_data.clone(),
            form_attrs: attributes! {
                action=(delete_citation_data
                    .get()
                    .map(
                        |d| Some(
                                href!(
                                    remove_citation,
                                    taxa::taxon_id: taxon.id,
                                    cleaning_id: cleaning_procedure.id,
                                    citation_id: d.0,
                                ),
                            ),
                    ))
                method="POST"
            },
            remove_citation_dialog_context(
                remove_citation_dialog_context_item(
                    item_term("Taxon")
                    item_value(
                        attrs: attributes! { class="latin" },
                        (&taxon.complete_name)
                    )
                )
                remove_citation_dialog_context_item(
                    item_term("Cleaning Procedure")
                    item_value(
                        (format!(
                            "{}: {}",
                            cleaning_procedure.id,
                            cleaning_procedure.name,
                        ))
                    )
                )
            )
        )
    })
}

#[derive(Debug, Deserialize)]
struct AddCitationParams {
    citation_id: u64,
}

#[route(POST "/taxa/{taxon_id}/cleaning/{cleaning_id}/citations")]
pub async fn add_citation(
    cx: &Cx,
    Form(params): Form<AddCitationParams>,
) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonCleaningEdit).await?;

    let taxon_id = *taxa::taxon_id(cx)?;
    let cleaning_id = *cleaning_id(cx)?;
    CleaningProcedureCitation::create()
        .cleaning_id(cleaning_id)
        .citation_id(params.citation_id)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(
        href!(modify, taxa::taxon_id: taxon_id, cleaning_id: cleaning_id).resolve(cx),
    ))
}

#[route(POST "/taxa/{taxon_id}/cleaning/{cleaning_id}/citations/{citation_id}/delete")]
pub async fn remove_citation(cx: &Cx) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonCleaningEdit).await?;
    let taxon_id = *taxa::taxon_id(cx)?;
    let cleaning_id = *cleaning_id(cx)?;
    let citation_id = *citation_id(cx)?;
    CleaningProcedureCitation::delete_by_citation_id_and_cleaning_id(
        &mut db(cx),
        citation_id,
        cleaning_id,
    )
    .await?;
    Ok(see_other(
        href!(modify, taxa::taxon_id: taxon_id, cleaning_id: cleaning_id).resolve(cx),
    ))
}
