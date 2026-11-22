use libpropagation::{
    auth::PermissionCode, citation::TaxonPropagationProcedureCitation,
    propagation::PropagationProcedure, taxonomy::TaxonPropagationProcedure,
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
    view::{Attributes, View, attributes, class, component, element_id, view},
};

use crate::{
    Subtitle,
    actions::{Action, ActionItemType},
    assets::mdi,
    components::{
        action_menu::*,
        alert_dialog::*,
        badge::{BadgeVariant, badge},
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
    context::{db, require_user_with_permission},
    flash::{self, FlashMessage},
    taxa,
};

path_param!(pub propagation_id: u64 else bad_request);
path_param!(pub citation_id: u64 else bad_request);

/// Shows a 0-10 confidence score as a labelled meter.
#[component]
async fn confidence_meter(
    cx: &Cx,
    confidence: Option<u8>,
    #[default] mut attrs: Attributes,
) -> topcoat::Result<impl View> {
    let value = confidence.unwrap_or_default().min(10);
    let (label, label_class) = match confidence {
        None => ("Not rated", "text-muted-foreground"),
        Some(0..=3) => ("Low confidence", "text-muted-foreground"),
        Some(4..=6) => ("Moderate confidence", "text-muted-foreground"),
        Some(7..=8) => ("Good confidence", "text-primary"),
        Some(9..=10) => ("High confidence", "text-primary"),
        Some(_) => ("Not rated", "text-muted-foreground"),
    };
    let aria_label = confidence.map_or_else(
        || "Confidence: not rated".to_string(),
        |c| format!("Confidence: {c} out of 10"),
    );

    Ok(view! {
        <div class=(class!("flex flex-col gap-2", attrs.remove("class"))) (attrs)>
            <div class="flex items-baseline justify-between gap-4">
                <span class="text-sm font-medium">"Confidence"</span>
                if let Some(confidence) = confidence {
                    <span class="text-sm font-semibold tabular-nums">
                        (format!("{confidence} / 10"))
                    </span>
                } else {
                    <span class="text-sm text-muted-foreground">"Not rated"</span>
                }
            </div>
            <div class="flex gap-1" role="img" aria-label=(aria_label)>
                for i in 0..10 {
                    if i < value {
                        <span class="h-2 flex-1 rounded-full bg-primary"></span>
                    } else {
                        <span class="h-2 flex-1 rounded-full bg-foreground/10"></span>
                    }
                }
            </div>
            <div class=(class!("text-xs", label_class))>(label)</div>
        </div>
    })
}

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
    let tp = taxon_propagation_procedure(cx).await?;
    let delete_ids = (tp.taxon_id, tp.propagation_id);
    let delete_dialog = signal(cx, || None::<(u64, u64)>);

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
                    delete_dialog.set(Some(delete_ids));
                })
                class="text-destructive"
            },
            item_type: ActionItemType::Button,
        },
    ];

    Ok(view! {
        let procedure = tp.propagation.get();
        let taxon = tp.taxon.get();
        <div class="flex flex-col gap-6">
            <section class="flex flex-col gap-4">
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
                        breadcrumb_item(breadcrumb_page((&procedure.name)))
                    )
                )
                <div class="flex items-start gap-4">
                    <div class="flex min-w-0 flex-col">
                        <div class="flex flex-wrap items-center gap-3">
                            <h1 class="mb-0">(&procedure.name)</h1>
                            badge(
                                variant: BadgeVariant::Secondary,
                                (procedure.r#type.to_string())
                            )
                        </div>
                        <p class="text-muted-foreground">
                            "Propagation guidance for "
                            <span class="latin text-foreground">
                                (&taxon.complete_name)
                            </span>
                        </p>
                    </div>
                    <div class="ms-auto">action_menu(actions: &menu_actions)</div>
                </div>
            </section>

            <div class="grid items-start gap-12 lg:grid-cols-3">
                card(
                    attrs: attributes! { class="lg:col-span-2" },
                    card_content(
                        <div class="flex flex-col gap-6">
                            <div class="flex items-start gap-3">
                                <div class="flex flex-col">
                                    <h2 class="mb-0">"General instructions"</h2>
                                    <div class="text-muted-foreground text-sm">
                                        "Shared guidance that applies to every taxon using this procedure."
                                    </div>
                                </div>
                            </div>
                            <div class="flex flex-col gap-6">
                                <p class="whitespace-pre-line text-lg leading-7">
                                    (&procedure.instructions)
                                </p>
                                if let Some(notes) = procedure.notes.as_ref() {
                                    <div>
                                        <h3>"Additional notes"</h3>
                                        <p class="whitespace-pre-line text-sm leading-6">
                                            (notes)
                                        </p>
                                    </div>
                                }
                                <div>
                                    <h3>"Citations"</h3>
                                    let general_citations = procedure.citations.get();
                                    if general_citations.is_empty() {
                                        <p class="text-sm text-muted-foreground">
                                            "No sources have been listed for this procedure."
                                        </p>
                                    } else {
                                        <div>
                                            citation_list(
                                                citations: general_citations.iter().collect()
                                            )
                                        </div>
                                    }
                                </div>
                            </div>
                        </div>
                    )
                )
                <div class="flex flex-col gap-6">
                    <div class="flex flex-col">
                        <h3 class="mb-0">"Notes for this taxon"</h3>
                        <div class="text-muted-foreground text-sm">
                            "Details that apply only to "
                            <span class="latin text-foreground">
                                (&taxon.complete_name)
                            </span>
                            "."
                        </div>
                    </div>
                    <div class="flex flex-col gap-6">
                        confidence_meter(confidence: tp.confidence)
                        <div>
                            <h4>"Notes"</h4>
                            if let Some(notes) = tp.notes.as_ref() {
                                <p
                                    class="mt-2 whitespace-pre-line text-sm leading-6 text-foreground/90"
                                >
                                    (notes)
                                </p>
                            } else {
                                <p class="mt-2 text-sm text-muted-foreground italic">
                                    "No taxon-specific notes have been added yet."
                                </p>
                            }
                        </div>
                        <div>
                            <h4>"Taxon-specific Citations"</h4>
                            let taxon_citations = tp.citation_links.get();
                            if taxon_citations.is_empty() {
                                <p class="mt-3 text-sm text-muted-foreground">
                                    "No sources are attached to this taxon."
                                </p>
                            } else {
                                <div>
                                    citation_list(
                                        citations: taxon_citations
                                            .iter()
                                            .map(|link| link.citation.get())
                                            .collect()
                                    )
                                </div>
                            }
                        </div>
                    </div>
                </div>
            </div>
        </div>
        delete_confirmation_dialog(delete_signal: delete_dialog.clone())
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
                                    (&proc.name)
                                )
                            )
                            breadcrumb_separator()
                            breadcrumb_item(breadcrumb_page("Edit"))
                        )
                    )
                    <h1>"Edit taxon-specific propagation guidance"</h1>
                    <p class="text-muted-foreground">
                        "Update the details that apply only to "
                        <span class="latin font-semibold text-foreground">
                            (&taxon.complete_name)
                        </span>
                        "."
                    </p>
                </hgroup>
            </section>
            cited_object_layout(
                cited_object_main_content(
                    card_content(
                        <h3>"Taxon-specific details"</h3>
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
                                    field_label("General procedure")
                                    <div class="flex flex-wrap items-center gap-2">
                                        (&proc.name)
                                        badge(
                                            variant: BadgeVariant::Secondary,
                                            (proc.r#type.to_string())
                                        )
                                    </div>
                                    field_description(
                                        "The general instructions and notes for this procedure are shared by all taxa and are not edited here."
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
                                    field_description(
                                        "How confident are you that this procedure applies to this taxon? 0 is unknown and 10 is certain."
                                    )
                                    if let Some(msg) = errors.confidence {
                                        field_error((msg))
                                    }
                                )
                                field(
                                    field_label("Taxon-specific notes")
                                    textarea(
                                        attrs: attributes! { name="notes" },
                                        (params.notes)
                                    )
                                    field_description(
                                        "Notes that apply only to this taxon. General notes belong on the shared procedure."
                                    )
                                    if let Some(msg) = errors.notes {
                                        field_error((msg))
                                    }
                                )
                                button("Save taxon-specific details")
                            )
                        </form>
                    )
                )
                cited_object_citation_section(
                    title: "Taxon-specific citations",
                    let citations = tp.citation_links.get();
                    for citation_link in citations {
                        let citation = citation_link.citation.get();
                        cited_object_citation_item(
                            let citation_data = (citation.id, citation.format_cse());
                            <span class="text-muted-foreground">
                                (citation.id)
                                "."
                            </span>
                            <span class="text-wrap">(citation.format_cse())</span>
                            button(
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Destructive,
                                attrs: attributes! {
                                    @click=$(|e: Event| {
                                        e.prevent_default();
                                        delete_citation_data.set(Some(citation_data));
                                    })
                                },
                                icon(data: mdi::REMOVE)
                            )
                        )
                    }
                    cited_object_citation_item(
                        button(
                            size: ButtonSize::Sm,
                            variant: ButtonVariant::Outline,
                            attrs: attributes! {
                                @click=$(|e: Event| {
                                    e.prevent_default();
                                    add_citation_dialog_open.set(true);
                                })
                            },
                            icon(data: mdi::ADD)
                        )
                        <span>"Add citation"</span>
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

/// Content for the dialog confirming deletion of a propagation procedure.
///
/// Renders nothing while `ids` is `None` (dialog closed). When set, the shard
/// re-renders with details of the procedure and a form that POSTs to
/// [`delete`]. The shard has its own endpoint, so it checks permissions itself.
#[shard]
pub async fn delete_confirmation(
    cx: &Cx,
    ids: Option<(u64, u64)>,
    delete_signal: Signal<Option<(u64, u64)>>,
) -> topcoat::Result<impl View> {
    let tp = match ids {
        Some((taxon_id, propagation_id)) => {
            require_user_with_permission(cx, PermissionCode::TaxonPropagationDelete).await?;
            Some(
                TaxonPropagationProcedure::filter_by_taxon_id_and_propagation_id(
                    taxon_id,
                    propagation_id,
                )
                .include(TaxonPropagationProcedure::fields().propagation())
                .include(TaxonPropagationProcedure::fields().taxon())
                .one()
                .exec(&mut db(cx))
                .await
                .ok_or_not_found()?,
            )
        }
        None => None,
    };

    Ok(view! {
        if let Some(tp) = tp {
            let taxon = tp.taxon.get();
            let procedure = tp.propagation.get();
            <section class="flex flex-col gap-3">
                <div>
                    item_term("Taxon")
                    item_value(
                        attrs: attributes! { class="latin" },
                        (&taxon.complete_name)
                    )
                </div>
                <div>
                    item_term("Propagation Procedure")
                    item_value((format!("{}: {}", procedure.id, procedure.name)))
                </div>
            </section>
            dialog_footer(
                button(
                    variant: ButtonVariant::Secondary,
                    attrs: attributes! {
                        @click=$(|e: Event| {
                            e.prevent_default();
                            delete_signal.set(None)
                        })
                    },
                    "Cancel"
                )
                <form
                    method="POST"
                    action=(href!(
                        delete,
                        taxa::taxon_id: tp.taxon_id,
                        propagation_id: tp.propagation_id,
                    ))
                    @submit=$(|_e: Event| delete_signal.set(None))
                >
                    button(variant: ButtonVariant::Destructive, "Delete")
                </form>
            )
        }
    })
}

/// Alert dialog for confirming deletion of a propagation procedure.
///
/// Opens client-side as soon as `delete_signal` is set; the
/// [`delete_confirmation`] shard populates the dialog's content (procedure
/// details and the delete form) from the ids in the signal.
#[component]
pub async fn delete_confirmation_dialog(
    cx: &Cx,
    delete_signal: Signal<Option<(u64, u64)>>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        alert_dialog(
            open: $(delete_signal.get().is_some()),
            dialog_content(
                dialog_header(
                    dialog_title("Delete this taxon propagation procedure?")
                    dialog_description("This action cannot be undone")
                )
                delete_confirmation(
                    ids: $(delete_signal.get()),
                    delete_signal: delete_signal.clone()
                )
            )
        )
    })
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
async fn add_propagation_procedure_form_fields(cx: &Cx) -> topcoat::Result<impl View> {
    let query = signal(cx, String::new);
    let query_value = query.get();

    let procs = if query_value.is_empty() {
        Vec::new()
    } else {
        PropagationProcedure::filter(
            PropagationProcedure::fields()
                .name()
                .like(format!("%{query_value}%")),
        )
        .exec(&mut db(cx))
        .await?
    };

    Ok(view! {
        field_set(
            field(
                input(
                    attrs: attributes! {
                        type="text"
                        name="query"
                        value=(&query_value)
                        placeholder="Search for an existing propagation procedure"
                        @input=$(|e: Event| query.set(e.target.value))
                    }
                )
            )
            field(
                if query_value.is_empty() {
                    <div class="text-muted-foreground">
                        "Start typing to search for a propagation procedure"
                    </div>
                } else {
                    <div>(format!("{} results shown", procs.len()))</div>
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
                }
            )
        )
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
            open: $(open_sig.get()),
            dialog_content(
                dialog_header(
                    dialog_title("Add a new propagation procedure to this taxon")
                )
                <form id=(&form_id) (form_attrs)>
                    add_propagation_procedure_form_fields()
                </form>
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
