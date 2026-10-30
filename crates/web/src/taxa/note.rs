use libpropagation::{
    auth::PermissionCode,
    citation::TaxonNoteCitation,
    taxonomy::{Taxon, TaxonNote, TaxonNoteCategory},
};
use serde::Deserialize;
use strum::IntoEnumIterator;
use topcoat::{
    context::{Cx, try_request_context},
    icon::icon,
    router::{
        Body, Method,
        content::Form,
        error::{RouterErrorExt, SeeOther, rewrite, see_other},
        href, page, path_param, route,
    },
    runtime::{Event, Signal, signal},
    view::{Attributes, Child, View, attributes, component, element_id, view},
};
use tracing::trace;

use crate::{
    actions::{Action, ActionItemType, permitted_actions},
    assets::mdi,
    components::{
        action_menu::action_menu,
        alert_dialog::alert_dialog,
        breadcrumb::*,
        button::*,
        card::*,
        citations::citation_list,
        cited_object::*,
        dialog::*,
        field::*,
        input::*,
        pn::{item_term, item_value},
        select::select,
        textarea::textarea,
    },
    context::{current_user, db, require_user_with_permission},
    taxa::{self, TaxonId},
};

path_param!(pub note_id: u64, error = bad_request);
path_param!(pub citation_id: u64, error = bad_request);

#[page("/taxa/{taxon_id}/note/{note_id}")]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let mut db = db(cx);
    let taxon_id = *path_param::<TaxonId>(cx)?;
    let note_id = *path_param::<NoteId>(cx)?;
    let note = TaxonNote::filter_by_id(note_id)
        .include(TaxonNote::fields().taxon())
        .include(TaxonNote::fields().citation_links().citation())
        .one()
        .exec(&mut db)
        .await
        .ok_or_not_found()?;

    let delete_dialog_open = signal(cx, || false);

    trace!(?note);

    let menu_actions = [
        Action {
            name: "Edit".to_string(),
            permission: Some(PermissionCode::TaxonNoteEdit),
            attrs: attributes! { href=(href!(edit, TaxonId(taxon_id), NoteId(note_id))) },
            item_type: ActionItemType::Link,
        },
        Action {
            name: "Delete".to_string(),
            permission: Some(PermissionCode::TaxonNoteDelete),
            attrs: attributes! {
                class="text-destructive"
                @click=$(|e: Event| {
                    e.prevent_default();
                    delete_dialog_open.set(true)
                })
            },
            item_type: ActionItemType::Button,
        },
    ];

    Ok(view! {
        let taxon = note.taxon.get();
        <div class="flex flex-col gap-4">
            <section>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(taxa::details, TaxonId(taxon.id))) },
                                (&taxon.complete_name)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page((format!("Note {}", note.id))))
                    )
                )
                <h1 class="flex items-center">
                    (&note.title)
                    let actions = permitted_actions(
                        current_user(cx).await,
                        &menu_actions,
                    );
                    if !actions.is_empty() {
                        <div class="ms-auto">action_menu(actions: &actions)</div>
                    }
                </h1>
                <div class="text-2xl">(note.text)</div>
            </section>
            <section>
                citation_list(
                    citations: note
                        .citation_links
                        .get()
                        .iter()
                        .map(|cl| cl.citation.get())
                        .collect()
                )
            </section>
        </div>
        alert_dialog(
            open: delete_dialog_open.get(),
            dialog_content(
                dialog_header(
                    dialog_title("Delete this taxon note?")
                    dialog_description("This action cannot be undone")
                )
                dialog_footer(
                    button(
                        attrs: attributes! {
                            @click=$(|e: topcoat::runtime::Event| {
                                e.prevent_default();
                                delete_dialog_open.set(false)
                            })
                        },
                        "Cancel"
                    )
                    <form
                        action=(href!(delete, TaxonId(taxon_id), NoteId(note_id)))
                        method="POST"
                    >
                        button(
                            variant: ButtonVariant::Destructive,
                            attrs: attributes! {},
                            "Delete"
                        )
                    </form>
                )
            )
        )
    })
}

#[derive(Deserialize, Clone, Debug)]
struct NoteParams {
    title: String,
    category: TaxonNoteCategory,
    text: String,
}

impl From<&TaxonNote> for NoteParams {
    fn from(value: &TaxonNote) -> Self {
        Self {
            title: value.title.clone(),
            category: value.category,
            text: value.text.clone(),
        }
    }
}

impl NoteParams {
    fn validate(&self) -> Option<NoteErrors> {
        let mut errors = NoteErrors::default();
        if self.title.trim().is_empty() {
            errors.title = Some("Title is required".to_string());
        }

        if self.text.trim().is_empty() {
            errors.text = Some("Note body is required".to_string())
        }

        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }
}

#[derive(Clone, Debug, Default)]
struct NoteErrors {
    title: Option<String>,
    category: Option<String>,
    text: Option<String>,
}

impl NoteErrors {
    fn is_empty(&self) -> bool {
        self.title.is_none() && self.category.is_none() && self.text.is_none()
    }
}

#[derive(Clone, Debug)]
struct TaxonNoteRejected {
    form: NoteParams,
    errors: NoteErrors,
}

#[component]
pub async fn note_form(
    cx: &Cx,
    #[default] attrs: Attributes,
    note: Option<&TaxonNote>,
    child: Child<'_>,
) -> topcoat::Result<impl View> {
    let rejected = try_request_context::<TaxonNoteRejected>(cx);
    let db_params: Option<NoteParams> = note.map(|n| n.into());
    let errors = rejected.map(|rej| &rej.errors);
    let params = rejected.map(|rej| rej.form.clone()).or(db_params);

    Ok(view! {
        <form (attrs)>
            field_set(
                field(
                    field_label("Title")
                    input(
                        attrs: attributes! {
                            name="title"
                            type="text"
                            value=(params.as_ref().map(|p| &p.title))
                        }
                    )
                    if let Some(msg) = errors.and_then(|e| e.title.as_ref()) {
                        field_error((msg))
                    }
                )
                field(
                    field_label("Category")
                    select(
                        attrs: attributes! { name="category" },
                        for category in TaxonNoteCategory::iter() {
                            let code: &'static str = category.into();
                            <option
                                selected=(params.as_ref().map(|p| p.category == category))
                                value=(code)
                            >
                                (category.to_string())
                            </option>
                        }
                    )
                    if let Some(msg) = errors.and_then(|e| e.category.as_ref()) {
                        field_error((msg))
                    }
                )
                field(
                    field_label("Text")
                    textarea(
                        attrs: attributes! { name="text" },
                        (params.as_ref().map(|p| &p.text))
                    )
                    if let Some(msg) = errors.and_then(|e| e.text.as_ref()) {
                        field_error((msg))
                    }
                )
            )
            (child)
        </form>
    })
}

#[page("/taxa/{taxon_id}/note/{note_id}/edit")]
pub async fn edit(cx: &Cx) -> topcoat::Result<impl View> {
    _ = require_user_with_permission(cx, PermissionCode::TaxonNoteEdit).await?;
    let mut db = db(cx);
    let taxon_id = *path_param::<TaxonId>(cx)?;
    let note_id = *path_param::<NoteId>(cx)?;
    let note = TaxonNote::filter_by_id(note_id)
        .include(TaxonNote::fields().taxon())
        .include(TaxonNote::fields().citation_links().citation())
        .one()
        .exec(&mut db)
        .await
        .ok_or_not_found()?;
    let form_id = element_id(cx);
    let add_citation_dialog_open = signal(cx, || false);
    let remove_citation_data = signal(cx, || None::<(u64, String)>);

    Ok(view! {
        let taxon = note.taxon.get();
        <section>
            <hgroup>
                breadcrumb(
                    breadcrumb_list(
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! { href=(href!(taxa::details, TaxonId(taxon.id))) },
                                (&taxon.complete_name)
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(
                            breadcrumb_link(
                                attrs: attributes! {
                                    href=(href!(details, TaxonId(taxon.id), NoteId(note_id)))
                                },
                                (format!("Note {}", note.id))
                            )
                        )
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page(("Edit")))
                    )
                )
                <h1>"Edit Note"</h1>
            </hgroup>
        </section>
        cited_object_layout(
            cited_object_main_content(
                card_header(card_title("Note Details"))
                card_content(
                    note_form(
                        attrs: attributes! {
                            class="flex flex-col gap-6"
                            id=(&form_id)
                            action=(href!(update, TaxonId(taxon_id), NoteId(note_id)))
                            method="POST"
                        },
                        note: Some(&note),
                        button(
                            attrs: attributes! { form=(&form_id) type="submit" },
                            "Update"
                        )
                    )
                )
            )
            cited_object_citation_section(
                for citation_link in note.citation_links.get() {
                    let citation = citation_link.citation.get();
                    cited_object_citation_item(
                        let citation_data = (citation.id, citation.format_cse());
                        <a
                            class="text-destructive"
                            href="#"
                            @click=$(|e: Event| {
                                e.prevent_default();
                                remove_citation_data.set(Some(citation_data));
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
        add_citation_dialog(
            open_sig: add_citation_dialog_open.clone(),
            form_attrs: attributes! {
                method="POST"
                action=(href!(add_citation, TaxonId(taxon_id), NoteId(note_id)))
            }
        )
        remove_citation_dialog(
            remove_signal: remove_citation_data.clone(),
            form_attrs: attributes! {
                action=(remove_citation_data
                    .get()
                    .map(
                        |d| Some(
                                href!(
                                    remove_citation,
                                    TaxonId(taxon_id),
                                    NoteId(note_id),
                                    CitationId(d.0),
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
                    item_term("Note")
                    item_value((format!("{}: {}", note.id, note.title)))
                )
            )
        )
    })
}

#[route(POST "/taxa/{taxon_id}/note/{note_id}")]
pub async fn update(cx: &Cx, Form(params): Form<NoteParams>) -> topcoat::Result<SeeOther> {
    _ = require_user_with_permission(cx, PermissionCode::TaxonNoteEdit).await?;
    let mut db = db(cx);
    let taxon_id = *path_param::<TaxonId>(cx)?;
    let note_id = *path_param::<NoteId>(cx)?;
    _ = TaxonNote::get_by_id(&mut db, note_id)
        .await
        .ok_or_not_found()?;
    if let Some(errors) = params.validate() {
        Err(rewrite(
            href!(edit, TaxonId(taxon_id), NoteId(note_id)).resolve(cx),
            Body::empty(),
        )
        .method(Method::GET)
        .with(TaxonNoteRejected {
            form: params,
            errors,
        })
        .into())
    } else {
        TaxonNote::update_by_id(note_id)
            .title(params.title)
            .category(params.category)
            .text(params.text)
            .exec(&mut db)
            .await?;
        Ok(see_other(
            href!(details, TaxonId(taxon_id), NoteId(note_id)).resolve(cx),
        ))
    }
}

#[page("/taxa/{taxon_id}/note/new")]
pub async fn create(cx: &Cx) -> topcoat::Result<impl View> {
    _ = require_user_with_permission(cx, PermissionCode::TaxonNoteCreate).await?;
    let taxon_id = *path_param::<TaxonId>(cx)?;
    let form_id = element_id(cx);
    let taxon = Taxon::get_by_id(&mut db(cx), taxon_id).await?;

    Ok(view! {
        breadcrumb(
            breadcrumb_list(
                breadcrumb_item(
                    breadcrumb_link(
                        attrs: attributes! { href=(href!(taxa::details, TaxonId(taxon.id))) },
                        (&taxon.complete_name)
                    )
                )
                breadcrumb_separator()
                breadcrumb_item(breadcrumb_page(("New Note")))
            )
        )
        <h1>"Create new note"</h1>
        note_form(
            attrs: attributes! {
                class="flex flex-col gap-6"
                id=(&form_id)
                action=(href!(do_create, TaxonId(taxon_id)))
                method="POST"
            },
            note: None,
            button(attrs: attributes! { form=(&form_id) type="submit" }, "Create")
        )
    })
}

#[route(POST "/taxa/{taxon_id}/note/new")]
pub async fn do_create(cx: &Cx, Form(params): Form<NoteParams>) -> topcoat::Result<SeeOther> {
    _ = require_user_with_permission(cx, PermissionCode::TaxonNoteCreate).await?;
    let mut db = db(cx);
    let taxon_id = *path_param::<TaxonId>(cx)?;
    let new_note = TaxonNote::create()
        .taxon_id(taxon_id)
        .title(params.title)
        .category(params.category)
        .text(params.text)
        .exec(&mut db)
        .await?;
    Ok(see_other(
        href!(details, TaxonId(taxon_id), NoteId(new_note.id)).resolve(cx),
    ))
}

#[route(POST "/taxa/{taxon_id}/note/{note_id}/delete")]
pub async fn delete(cx: &Cx) -> topcoat::Result<SeeOther> {
    _ = require_user_with_permission(cx, PermissionCode::TaxonNoteDelete).await?;
    let mut db = db(cx);
    let taxon_id = *path_param::<TaxonId>(cx)?;
    let note_id = *path_param::<NoteId>(cx)?;
    _ = TaxonNote::get_by_id(&mut db, note_id)
        .await
        .ok_or_not_found()?;
    TaxonNote::delete_by_id(&mut db, note_id).await?;
    Ok(see_other(
        href!(taxa::details, TaxonId(taxon_id)).resolve(cx),
    ))
}

#[derive(Debug, Deserialize)]
struct AddCitationParams {
    citation_id: u64,
}

#[route(POST "/taxa/{taxon_id}/note/{note_id}/citations")]
pub async fn add_citation(
    cx: &Cx,
    Form(params): Form<AddCitationParams>,
) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonCleaningEdit).await?;

    let taxon_id = *path_param::<TaxonId>(cx)?;
    let note_id = *path_param::<NoteId>(cx)?;
    TaxonNoteCitation::create()
        .note_id(note_id)
        .citation_id(params.citation_id)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(
        href!(edit, TaxonId(taxon_id), NoteId(note_id)).resolve(cx),
    ))
}

#[route(POST "/taxa/{taxon_id}/note/{note_id}/citations/{citation_id}/delete")]
pub async fn remove_citation(cx: &Cx) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonCleaningEdit).await?;
    let taxon_id = *path_param::<TaxonId>(cx)?;
    let note_id = *path_param::<NoteId>(cx)?;
    let citation_id = *path_param::<CitationId>(cx)?;
    TaxonNoteCitation::delete_by_citation_id_and_note_id(&mut db(cx), citation_id, note_id).await?;
    Ok(see_other(
        href!(edit, TaxonId(taxon_id), NoteId(note_id)).resolve(cx),
    ))
}
