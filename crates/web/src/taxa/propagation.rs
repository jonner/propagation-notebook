use libpropagation::{
    auth::PermissionCode,
    citation::{Citation, TaxonPropagationProcedureCitation},
    propagation::PropagationProcedure,
    taxonomy::TaxonPropagationProcedure,
};
use serde::Deserialize;
use topcoat::{
    context::{Cx, memoize, try_context},
    icon::icon,
    metadata::Metadata,
    router::{
        Body, Method, StatusCode,
        content::Form,
        error::{RouterErrorExt, SeeOther, rewrite, see_other},
        href, page, path_param, route,
    },
    runtime::{Event, Signal, shard, signal},
    view::{Attributes, View, attributes, component, element_id, view},
};

use crate::{
    Subtitle,
    actions::{Action, ActionItemType, permitted_actions},
    assets::mdi,
    components::{
        action_menu::*,
        alert_dialog::*,
        badge::badge,
        breadcrumb::*,
        button::*,
        card::*,
        citations::citation_list,
        cited_object::*,
        dialog::*,
        field::*,
        input::*,
        label::*,
        pn::{item_term, item_value},
        radio_group::*,
        select::*,
        textarea::*,
    },
    context::{current_user, db, require_user_with_permission},
    flash::{self, FlashMessage},
    taxa,
};

path_param!(pub propagation_id: u64 else bad_request);
path_param!(pub citation_id: u64 else bad_request);

#[memoize(as_ref)]
async fn taxon_propagation_procedure(cx: &Cx) -> topcoat::Result<TaxonPropagationProcedure> {
    let taxon_id = *taxa::taxon_id(cx)?;
    let propagation_id = *propagation_id(cx)?;
    Ok(
        TaxonPropagationProcedure::filter_by_taxon_id_and_propagation_id(taxon_id, propagation_id)
            .include(
                TaxonPropagationProcedure::fields()
                    .propagation()
                    .citations(),
            )
            .include(TaxonPropagationProcedure::fields().taxon())
            .include(
                TaxonPropagationProcedure::fields()
                    .citation_links()
                    .citation(),
            )
            .one()
            .exec(&mut db(cx))
            .await
            .ok_or_not_found()?,
    )
}
#[page(
    "/taxa/{taxon_id}/propagation/{propagation_id}",
    with(Metadata::from_fn(
        async |cx| {
            let tp = taxon_propagation_procedure(cx).await?;
            Ok(
                Metadata::new().subtitle(
                    format!(
                        "Propagation procedure {} for {}",
                        tp.propagation_id,
                        tp.taxon.get().complete_name,
                    ),
                ),
            )
        },
    ))
)]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let delete_dialog_open = signal(cx, || false);
    let tp = taxon_propagation_procedure(cx).await?;

    let menu_actions = [
        Action {
            name: "Edit".to_string(),
            permission: Some(PermissionCode::TaxonPropagationEdit),
            attrs: attributes! {
                href=(href!(
                    modify,
                    taxa::taxon_id: tp.taxon_id,
                    propagation_id: tp.propagation_id,
                ))
            },
            item_type: ActionItemType::Link,
        },
        Action {
            name: "Delete".to_string(),
            permission: Some(PermissionCode::TaxonPropagationDelete),
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
        let procedure = tp.propagation.get();
        let taxon = tp.taxon.get();
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
                        breadcrumb_item(breadcrumb_page("Propagation Procedures"))
                        breadcrumb_separator()
                        breadcrumb_item(breadcrumb_page((tp.propagation_id)))
                    )
                )
                <h1 class="flex items-center">
                    (&procedure.name)
                    let permitted_actions = permitted_actions(
                        current_user(cx).await,
                        &menu_actions,
                    );
                    if !permitted_actions.is_empty() {
                        <div class="ms-auto">
                            action_menu(actions: &permitted_actions)
                        </div>
                    }
                </h1>
                <div>badge((procedure.r#type.to_string()))</div>
            </hgroup>
            <section>
                <h2>"Instructions"</h2>
                <div class="text-2xl">(&procedure.instructions)</div>
            </section>
            <section>
                <h2>"Confidence"</h2>
                <div>
                    (tp.confidence.map(|v| v.to_string()).unwrap_or("Unknown".into()))
                </div>
            </section>
            <section>
                <h2>"Notes"</h2>
                <div>(procedure.notes.as_deref().unwrap_or_default())</div>
                if let Some(taxon_notes) = tp.notes.as_ref() {
                    <div>(taxon_notes)</div>
                }
            </section>
            <section>
                <h2>"Citations"</h2>
                <div>
                    let citations: Vec<&Citation> = tp
                        .citation_links
                        .get()
                        .iter()
                        .map(|cl| cl.citation.get())
                        .chain(procedure.citations.get().iter())
                        .collect();
                    if !citations.is_empty() {
                        citation_list(citations: citations)
                    }
                </div>
            </section>
        </div>
        alert_dialog(
            open: delete_dialog_open.get(),
            dialog_content(
                dialog_header(
                    dialog_title("Delete this taxon propagation procedure?")
                    dialog_description("This action is not reversible")
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
                        action=(href!(
                            delete,
                            taxa::taxon_id: tp.taxon_id,
                            propagation_id: tp.propagation_id,
                        ))
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

#[page(
    "/taxa/{taxon_id}/propagation/{propagation_id}/edit",
    with(Metadata::from_fn(
        async |cx| {
            let tp = taxon_propagation_procedure(cx).await?;
            Ok(
                Metadata::new().subtitle(
                    format!(
                        "Edit propagation procedure {} for {}",
                        tp.propagation_id,
                        tp.taxon.get().complete_name,
                    ),
                ),
            )
        },
    ))
)]
pub async fn modify(cx: &Cx) -> topcoat::Result<impl View> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonPropagationEdit).await?;
    let add_citation_dialog_open = signal(cx, || false);
    let delete_citation_data = signal(cx, || None::<(u64, String)>);
    let tp = taxon_propagation_procedure(cx).await?;
    let rejected = try_context::<TaxonPropagationRejected>(cx);
    let errors = rejected.map(|r| r.errors.clone()).unwrap_or_default();
    let params = rejected
        .map(|r| r.form.clone())
        .unwrap_or_else(|| tp.into());

    Ok(view! {
        let proc = tp.propagation.get();
        if rejected.is_some() {
            (StatusCode::UNPROCESSABLE_ENTITY)
        }
        let taxon = tp.taxon.get();
        <div class="flex flex-col gap-6">
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
                            breadcrumb_item(breadcrumb_page("Propagation Procedures"))
                            breadcrumb_separator()
                            breadcrumb_item(
                                breadcrumb_link(
                                    attrs: attributes! {
                                        href=(href!(
                                            details,
                                            taxa::taxon_id: tp.taxon_id,
                                            propagation_id: tp.propagation_id,
                                        ))
                                    },
                                    (tp.propagation_id)
                                )
                            )
                            breadcrumb_separator()
                            breadcrumb_item(breadcrumb_page("Edit"))
                        )
                    )
                    <h1>"Edit Propagation Procedure"</h1>
                </hgroup>
            </section>
            cited_object_layout(
                cited_object_main_content(
                    card_header(card_title("Taxon procedure details"))
                    card_content(
                        <form
                            action=(href!(
                                do_modify,
                                taxa::taxon_id: tp.taxon_id,
                                propagation_id: tp.propagation_id,
                            ))
                            method="POST"
                            class="flex flex-col gap-6"
                        >
                            field_set(
                                field(
                                    field_label("Propagation Procedure")
                                    <span class="text-muted-foreground">
                                        (proc.id)
                                        ": "
                                        (&proc.name)
                                    </span>
                                )
                            )
                            field(
                                field_label("Confidence")
                                select(
                                    attrs: attributes! { name="confidence" },
                                    <option></option>
                                    for i in 0..=10 {
                                        <option
                                            value=(i)
                                            if params.confidence == Some(i) {
                                                selected=(true)
                                            }
                                        >
                                            (i)
                                        </option>
                                    }
                                )
                                if let Some(msg) = errors.confidence {
                                    field_error((msg))
                                }
                            )
                            field(
                                field_label("Taxon-specific procedure notes")
                                textarea(
                                    attrs: attributes! { name="notes" },
                                    (params.notes)
                                )
                                if let Some(msg) = errors.notes {
                                    field_error((msg))
                                }
                            )
                            button("Update")
                        </form>
                    )
                )
                cited_object_citation_section(
                    let citations = tp.citation_links.get();
                    for citation_link in citations {
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
        </div>
        let citation_ids = tp
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
                    taxa::taxon_id: tp.taxon_id,
                    propagation_id: tp.propagation_id,
                ))
            }
        )
        remove_citation_dialog(
            remove_signal: delete_citation_data.clone(),
            form_attrs: attributes! {
                method="POST"
                action=(delete_citation_data
                    .get()
                    .map(
                        |d|
                            href!(
                                remove_citation,
                                taxa::taxon_id: tp.taxon_id,
                                propagation_id: tp.propagation_id,
                                citation_id: d.0,
                            ),
                    ))
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
                    item_term("Propagation Procedure")
                    item_value((format!("{}: {}", proc.id, proc.name)))
                )
            )
        )
    })
}

struct TaxonPropagationRejected {
    form: TaxonPropagationParams,
    errors: TaxonPropagationParamErrors,
}

#[derive(Deserialize, Debug, Default, Clone)]
struct TaxonPropagationParams {
    pub confidence: Option<u8>,
    pub notes: Option<String>,
}

impl From<&TaxonPropagationProcedure> for TaxonPropagationParams {
    fn from(value: &TaxonPropagationProcedure) -> Self {
        Self {
            confidence: value.confidence,
            notes: value.notes.clone(),
        }
    }
}

impl TaxonPropagationParams {
    fn validate(&self) -> Option<TaxonPropagationParamErrors> {
        let mut errors = TaxonPropagationParamErrors::default();

        if let Some(confidence) = self.confidence
            && confidence > 10
        {
            errors.confidence = Some("Confidence should be between 0 and 10".to_string());
        }
        if !errors.is_empty() {
            Some(errors)
        } else {
            None
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
struct TaxonPropagationParamErrors {
    pub confidence: Option<String>,
    pub notes: Option<String>,
}

impl TaxonPropagationParamErrors {
    fn is_empty(&self) -> bool {
        self.confidence.is_none() && self.notes.is_none()
    }
}

#[route(POST "/taxa/{taxon_id}/propagation/{propagation_id}")]
pub async fn do_modify(
    cx: &Cx,
    Form(params): Form<TaxonPropagationParams>,
) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonPropagationEdit).await?;
    let taxon_id = *taxa::taxon_id(cx)?;
    let propagation_id = *propagation_id(cx)?;
    match params.validate() {
        Some(errors) => Err(rewrite(
            href!(modify, taxa::taxon_id: taxon_id, propagation_id: propagation_id).resolve(cx),
            Body::empty(),
        )
        .with(TaxonPropagationRejected {
            form: params.clone(),
            errors,
        })
        .method(Method::GET)
        .into()),
        None => {
            TaxonPropagationProcedure::upsert_by_taxon_id_and_propagation_id(
                taxon_id,
                propagation_id,
            )
            .confidence(params.confidence)
            .notes(params.notes)
            .exec(&mut db(cx))
            .await?;
            Ok(see_other(
                href!(details, taxa::taxon_id: taxon_id, propagation_id: propagation_id)
                    .resolve(cx),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct AddCitationParams {
    citation_id: u64,
}

#[route(POST "/taxa/{taxon_id}/propagation/{propagation_id}/citations")]
pub async fn add_citation(
    cx: &Cx,
    Form(params): Form<AddCitationParams>,
) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonPropagationEdit).await?;

    let taxon_id = *taxa::taxon_id(cx)?;
    let propagation_id = *propagation_id(cx)?;
    TaxonPropagationProcedureCitation::create()
        .taxon_id(taxon_id)
        .propagation_id(propagation_id)
        .citation_id(params.citation_id)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(
        href!(modify, taxa::taxon_id: taxon_id, propagation_id: propagation_id).resolve(cx),
    ))
}

#[route(
    POST "/taxa/{taxon_id}/propagation/{propagation_id}/citations/{citation_id}/delete"
)]
pub async fn remove_citation(cx: &Cx) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonPropagationEdit).await?;
    let taxon_id = *taxa::taxon_id(cx)?;
    let propagation_id = *propagation_id(cx)?;
    let citation_id = *citation_id(cx)?;
    TaxonPropagationProcedureCitation::delete_by_citation_id_and_propagation_id_and_taxon_id(
        &mut db(cx),
        citation_id,
        propagation_id,
        taxon_id,
    )
    .await?;
    Ok(see_other(
        href!(modify, taxa::taxon_id: taxon_id, propagation_id: propagation_id).resolve(cx),
    ))
}

#[route(POST "/taxa/{taxon_id}/propagation/{propagation_id}/delete")]
pub async fn delete(cx: &Cx) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonPropagationDelete).await?;
    let taxon_id = *taxa::taxon_id(cx)?;
    let propagation_id = *propagation_id(cx)?;
    TaxonPropagationProcedure::delete_by_taxon_id_and_propagation_id(
        &mut db(cx),
        taxon_id,
        propagation_id,
    )
    .await?;
    Ok(see_other(
        href!(taxa::details, taxa::taxon_id: taxon_id).resolve(cx),
    ))
}
#[derive(Debug, Deserialize)]
struct AddPropagationParams {
    propagation_id: u64,
}

#[route(POST "/taxa/{taxon_id}/propagation/new")]
pub async fn add_propagation_procedure(
    cx: &Cx,
    Form(params): Form<AddPropagationParams>,
) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::TaxonPropagationCreate).await?;

    let taxon_id = *taxa::taxon_id(cx)?;
    flash::push_message(
        cx,
        match TaxonPropagationProcedure::create()
            .taxon_id(taxon_id)
            .propagation_id(params.propagation_id)
            .exec(&mut db(cx))
            .await
        {
            Ok(proc) => FlashMessage::Success(format!(
                "Added propagation procedure {} to taxon {}",
                proc.propagation_id, taxon_id
            )),
            Err(e) => {
                tracing::warn!(
                    "failed to add propagation procedure {} to taxon {taxon_id}: {e}",
                    params.propagation_id
                );
                FlashMessage::Failure(
                    "Failed to add propagation procedure to this taxon".to_string(),
                )
            }
        },
    )?;
    Ok(see_other(
        href!(taxa::details, taxa::taxon_id: taxon_id).resolve(cx),
    ))
}

#[shard]
async fn propagation_procedure_options(cx: &Cx, query: String) -> topcoat::Result<impl View> {
    let n_procs = PropagationProcedure::all()
        .count()
        .exec(&mut db(cx))
        .await?;
    let procs = if query.is_empty() {
        Vec::default()
    } else {
        PropagationProcedure::filter(
            PropagationProcedure::fields()
                .name()
                .like(format!("%{query}%")),
        )
        .exec(&mut db(cx))
        .await?
    };
    Ok(view! {
        <div>(format!("{} / {n_procs} results shown", procs.len()))</div>
        if !procs.is_empty() {
            radio_group(
                for proc in procs {
                    let id = format!("propagation-{}", proc.id);
                    <div class="flex items-center gap-2">
                        radio_group_item(
                            attrs: attributes! { id=(&id) name="propagation_id" value=(proc.id) }
                        )
                        label(attrs: attributes! { for=(&id) }, (&proc.name))
                    </div>
                }
            )
        }
        // <a class="inline-flex items-center gap-2" href=(href!(propagation::create))>
        //     icon(data: mdi::ADD)
        //     "Create a new propagation procedure"
        // </a>
    })
}

#[component]
async fn add_propagation_procedure_form(
    cx: &Cx,
    #[default] attrs: Attributes,
) -> topcoat::Result<impl View> {
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
                            placeholder="Search for an existing propagation procedure"
                            @input=$(|e: Event| query.set(e.target.value))
                        }
                    )
                )
                field(propagation_procedure_options(query: query.get()))
            )
        </form>
    })
}

#[component]
pub async fn add_propagation_procedure_dialog(
    cx: &Cx,
    open_sig: Signal<bool>,
    #[default] form_attrs: Attributes,
) -> topcoat::Result<impl View> {
    let form_id = element_id(cx);
    Ok(view! {
        dialog(
            open: open_sig.get(),
            dialog_content(
                dialog_header(
                    dialog_title("Add a new propagation procedure to this taxon")
                )
                add_propagation_procedure_form(
                    attrs: attributes! { id=(&form_id) (form_attrs) }
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
                        attrs: attributes! { form=(&form_id) },
                        "Add"
                    )
                )
            )
        )
    })
}
