use libpropagation::{
    auth::PermissionCode,
    taxonomy::{Taxon, TaxonNote, TaxonNoteCategory},
};
use serde::Deserialize;
use strum::IntoEnumIterator;
use topcoat::{
    context::{Cx, try_request_context},
    router::{
        Body, Method,
        content::Form,
        error::{RouterErrorExt, SeeOther, rewrite, see_other},
        href, page, path_param, route,
    },
    runtime::{Event, signal},
    view::{Attributes, Child, View, attributes, component, element_id, view},
};
use tracing::trace;

use crate::{
    actions::{Action, ActionItemType, permitted_actions},
    components::{
        action_menu::action_menu, alert_dialog::alert_dialog, breadcrumb::*, button::*,
        citations::citation_list, dialog::*, field::*, input::*, select::select,
        textarea::textarea,
    },
    context::{current_user, db, require_user_with_permission},
    taxa::{self, TaxonId},
};

path_param!(pub note_id: u64, error = bad_request);

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

    Ok(view! {
        let taxon = note.taxon.get();
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
        note_form(
            attrs: attributes! {
                class="flex flex-col gap-6"
                id=(&form_id)
                action=(href!(update, TaxonId(taxon_id), NoteId(note_id)))
                method="POST"
            },
            note: Some(&note),
            button(attrs: attributes! { form=(&form_id) type="submit" }, "Update")
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
