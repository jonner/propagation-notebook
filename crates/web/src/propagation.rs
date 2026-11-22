use libpropagation::{
    auth::PermissionCode,
    propagation::{ProcedureType, PropagationProcedure},
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use serde::Deserialize;
use strum::IntoEnumIterator;
use topcoat::{
    context::{Cx, memoize},
    icon::icon,
    metadata::Metadata,
    router::{
        Slot,
        content::Form,
        error::{SeeOther, see_other},
        href, layout, page, path_param, route,
    },
    runtime::signal,
    view::{Attributes, Child, View, attributes, class, component, view},
};
use tracing::trace;

use crate::{
    Subtitle,
    actions::{Action, ActionItemType, permitted_actions},
    assets::mdi,
    components::{
        action_menu::action_menu,
        alert_dialog::alert_dialog,
        button::*,
        card::*,
        citations::citation_list,
        dialog::*,
        field::*,
        input::*,
        pn::{item_term, item_value},
        select::*,
        textarea::*,
    },
    context::{current_user, db, require_user_with_permission},
    crumb::{Crumb, crumbs},
    flash::{self, FlashMessage, push_message},
    taxa,
};

path_param!(pub propagation_id: u64 else bad_request);

#[layout(
    "/admin/propagation",
    with(Crumb::new(
        async |cx| Ok(
                ("Propagation Procedures".to_string(), Some(href!(list).resolve(cx))),
            ),
    ))
)]
pub async fn propagation_admin_shell(cx: &Cx, slot: Slot<'_>) -> topcoat::Result<impl View> {
    Ok(view! { (slot) })
}

#[page("/admin/propagation", with(Metadata::new().subtitle("Propagation Procedures")))]
pub async fn list(cx: &Cx) -> topcoat::Result<impl View> {
    let _user = require_user_with_permission(cx, PermissionCode::PropagationEdit).await?;
    let mut db = db(cx);
    let procedures = PropagationProcedure::all().exec(&mut db).await?;
    trace!(?procedures);
    Ok(view! {
        crumbs()
        <h1>"Propagation Procedures"</h1>
        <ul>
            for p in procedures {
                <li class="flex items-baseline gap-2">
                    <span>
                        <a href=(href!(details, propagation_id: p.id))>(p.id)</a>
                    </span>
                    <span>
                        <span>(&p.name)</span>
                        <span class="inline-block mx-2">
                            <a
                                class=(button_variants(
                                    ButtonVariant::Secondary,
                                    ButtonSize::default(),
                                ))
                                href=(href!(modify, propagation_id: p.id))
                            >
                                icon(data: mdi::EDIT)
                            </a>
                        </span>
                    </span>
                </li>
            }
            <li>
                <a
                    href=(href!(new))
                    class=(button_variants(ButtonVariant::Primary, ButtonSize::Md))
                >
                    "New Propagation Procedure"
                </a>
            </li>
        </ul>
    })
}

#[derive(Deserialize, Debug, Clone)]
struct PropagationProcedureParams {
    #[serde(rename = "procedure_name")]
    name: String,
    #[serde(rename = "procedure_type")]
    r#type: ProcedureType,
    #[serde(rename = "procedure_instructions")]
    instructions: String,
    #[serde(rename = "procedure_notes")]
    notes: Option<String>,
}

#[route(POST "/admin/propagation/{propagation_id}")]
pub async fn do_modify(
    cx: &Cx,
    Form(params): Form<PropagationProcedureParams>,
) -> topcoat::Result<SeeOther> {
    let _user = require_user_with_permission(cx, PermissionCode::PropagationEdit).await?;
    let id = *propagation_id(cx)?;

    PropagationProcedure::update_by_id(id)
        .name(params.name)
        .r#type(params.r#type)
        .instructions(params.instructions)
        .notes(params.notes)
        .exec(&mut db(cx))
        .await?;
    push_message(
        cx,
        FlashMessage::Success(format!("Updated propagation procedure {id}")),
    )?;

    Ok(see_other(href!(details, propagation_id: id).resolve(cx)))
}

#[route(POST "/admin/propagation/new")]
pub async fn do_create(
    cx: &Cx,
    Form(params): Form<PropagationProcedureParams>,
) -> topcoat::Result<SeeOther> {
    let _user = require_user_with_permission(cx, PermissionCode::PropagationCreate).await?;

    let proc = PropagationProcedure::create()
        .name(params.name)
        .r#type(params.r#type)
        .instructions(params.instructions)
        .notes(params.notes)
        .exec(&mut db(cx))
        .await?;
    push_message(
        cx,
        FlashMessage::Success(format!("Created new propagation procedure {}", proc.id)),
    )?;

    Ok(see_other(
        href!(details, propagation_id: proc.id).resolve(cx),
    ))
}

#[component]
async fn propagation_procedure_form(
    cx: &Cx,
    procedure: Option<&PropagationProcedure>,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <form class=(class!("flex flex-col gap-6", attrs.remove("class"))) (attrs)>
            field_set(
                field(
                    field_label("Procedure Name")
                    input(
                        attrs: attributes! {
                            name="procedure_name"
                            type="text"
                            value=(procedure.map(|p| &p.name))
                        }
                    )
                )
                field(
                    field_label("Procedure type")
                    select(
                        attrs: attributes! { name="procedure_type" },
                        for t in ProcedureType::iter() {
                            <option value=(serde_plain::to_string(&t)?)>
                                (t.to_string())
                            </option>
                        }
                    )
                )
                field(
                    field_label("Instructions")
                    textarea(
                        attrs: attributes! { name="procedure_instructions" },
                        (procedure.map(|p| &p.instructions))
                    )
                )
                field(
                    field_label("Additional Notes")
                    textarea(
                        attrs: attributes! { name="procedure_notes" },
                        (procedure.and_then(|p| p.notes.as_ref()))
                    )
                )
            )
            (child)
        </form>
    })
}

#[page(
    "/admin/propagation/{propagation_id}",
    with(Metadata::from_fn(
        async |cx| {
            let id = propagation_id(cx)?;
            Ok(Metadata::new().subtitle(format!("Edit Propagation Procedure {id}")))
        },
    )),
    with(Crumb::new(async |cx| Ok((propagation_id(cx)?.to_string(), None))))
)]
pub async fn modify(cx: &Cx) -> topcoat::Result<impl View> {
    let _user = require_user_with_permission(cx, PermissionCode::PropagationEdit).await?;
    let id = *propagation_id(cx)?;
    let propagation = PropagationProcedure::get_by_id(&mut db(cx), id).await?;
    Ok(view! {
        <hgroup>
            crumbs()
            <h1>"Edit a Propagation Procedure"</h1>
        </hgroup>
        propagation_procedure_form(
            procedure: Some(&propagation),
            attrs: attributes! {
                method="POST"
                action=(href!(do_modify, propagation_id: id))
            },
            button("Update")
        )
    })
}

#[page(
    "/admin/propagation/new",
    with(Metadata::new().subtitle("New Propagation Procedure")),
    with(Crumb::new(async |_cx| Ok(("New".to_string(), None))))
)]
pub async fn new(cx: &Cx) -> topcoat::Result<impl View> {
    let _user = require_user_with_permission(cx, PermissionCode::PropagationCreate).await?;
    Ok(view! {
        <hgroup>
            crumbs()
            <h1>"New Propagation Procedure"</h1>
        </hgroup>
        propagation_procedure_form(
            procedure: None,
            attrs: attributes! { method="POST" action=(href!(do_create)) },
            button("Create")
        )
    })
}

#[memoize(as_ref)]
async fn propagation_procedure(cx: &Cx) -> topcoat::Result<PropagationProcedure> {
    let propagation_id = propagation_id(cx)?;
    Ok(PropagationProcedure::filter_by_id(propagation_id)
        .include(PropagationProcedure::fields().citations())
        .one()
        .exec(&mut db(cx))
        .await?)
}

#[page(
    "/propagation/{propagation_id}",
    with(Metadata::from_fn(
        async |cx| {
            let proc = propagation_procedure(cx).await?;
            Ok(Metadata::new().subtitle(&proc.name))
        },
    ))
)]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let procedure = propagation_procedure(cx).await?;
    // including the Taxa in the PropagationProcedure query is very slow, so fetch them separately
    let taxa = Taxon::filter(
        Taxon::fields().propagation_procedures().any(
            TaxonPropagationProcedure::fields()
                .propagation_id()
                .eq(procedure.id),
        ),
    )
    .include(
        Taxon::fields().propagation_procedures().filter(
            TaxonPropagationProcedure::fields()
                .propagation_id()
                .eq(procedure.id),
        ),
    )
    .exec(&mut db(cx))
    .await?;

    let delete_dialog_open = signal(cx, || false);
    let actions = [
        Action {
            name: "Edit".to_string(),
            permission: Some(PermissionCode::PropagationEdit),
            item_type: ActionItemType::Link,
            attrs: attributes! { href=(href!(modify, propagation_id: procedure.id)) },
        },
        Action {
            name: "Delete".to_string(),
            permission: Some(PermissionCode::PropagationDelete),
            item_type: ActionItemType::Button,
            attrs: attributes! {
                @click=$(|_e: topcoat::runtime::Event| {
                    delete_dialog_open.set(true);
                })
                class="text-destructive"
            },
        },
    ];
    Ok(view! {
        <div class="flex flex-col gap-6">
            <div class="flex">
                <h1>(&procedure.name)</h1>
                let actions = permitted_actions(current_user(cx).await, &actions);
                if !actions.is_empty() {
                    <div class="ms-auto">action_menu(actions: actions.as_slice())</div>
                }
            </div>
            <div class="grid gap-6 lg:grid-cols-3">
                <section class="lg:col-span-2">
                    card(
                        card_header(card_title("Procedure details"))
                        card_content(
                            <div class="flex flex-col gap-6">
                                <div>
                                    item_term("Type")
                                    item_value((procedure.r#type.to_string()))
                                </div>
                                <div>
                                    item_term("Notes")
                                    item_value(
                                        (procedure.notes.as_deref().unwrap_or_default())
                                    )
                                </div>
                                <div>
                                    item_term("Instructions")
                                    item_value(
                                        attrs: attributes! { class="text-lg leading-7" },
                                        (&procedure.instructions)
                                    )
                                </div>
                                <div>
                                    item_term("Citations")
                                    item_value(
                                        if !procedure.citations.get().is_empty() {
                                            citation_list(
                                                citations: procedure.citations.get().iter().collect()
                                            )
                                        } else {
                                            "None"
                                        }
                                    )
                                </div>
                            </div>
                        )
                    )
                </section>
                <section>
                    <h3>"Associated Taxa"</h3>
                    <div>
                        if !taxa.is_empty() {
                            <ul>
                                for taxon in taxa {
                                    <li class="flex items-baseline gap-2">
                                        <span class="latin">
                                            <a href=(href!(taxa::details, taxa::taxon_id: taxon.id))>
                                                (&taxon.complete_name)
                                            </a>
                                        </span>
                                    </li>
                                }
                            </ul>
                        } else {
                            "None"
                        }
                    </div>
                </section>
            </div>
        </div>
        alert_dialog(
            open: delete_dialog_open.get(),
            dialog_content(
                dialog_header(
                    dialog_title("Delete this procedure?")
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
                        action=(href!(delete_propagation, propagation_id: procedure.id))
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

#[route(POST "/admin/propagation/{propagation_id}/delete")]
pub async fn delete_propagation(cx: &Cx) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::PropagationDelete).await?;
    let id = *propagation_id(cx)?;
    match PropagationProcedure::delete_by_id(&mut db(cx), id).await {
        Ok(_) => {
            flash::push_message(
                cx,
                FlashMessage::Success(format!("Deleted propagation procedure {id}")),
            )?;
            Ok(see_other(href!(crate::home::index).resolve(cx)))
        }
        Err(e) => {
            flash::push_message(
                cx,
                FlashMessage::Failure(format!("Failed to delete propagation procedure : {e}")),
            )?;
            Err(see_other(href!(details, propagation_id: id).resolve(cx)).into())
        }
    }
}
