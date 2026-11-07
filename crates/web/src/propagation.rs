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
        content::Form,
        error::{SeeOther, see_other},
        href, page, path_param, route,
    },
    view::{Attributes, Child, View, attributes, class, component, view},
};
use tracing::trace;

use crate::{
    Subtitle, admin,
    assets::mdi,
    components::{
        breadcrumb::*,
        button::*,
        card::*,
        citations::citation_list,
        field::*,
        input::*,
        pn::{item_term, item_value},
        select::*,
        textarea::*,
    },
    context::{db, require_user_with_permission},
    flash::{FlashMessage, push_message},
    taxa,
};

path_param!(pub propagation_id: u64 else bad_request);

#[page("/admin/propagation", with(Metadata::new().subtitle("Propagation procedures")))]
pub async fn list(cx: &Cx) -> topcoat::Result<impl View> {
    let _user = require_user_with_permission(cx, PermissionCode::PropagationEdit).await?;
    let mut db = db(cx);
    let procedures = PropagationProcedure::all().exec(&mut db).await?;
    trace!(?procedures);
    Ok(view! {
        breadcrumb(
            breadcrumb_list(
                breadcrumb_item(
                    breadcrumb_link(
                        attrs: attributes! { href=(href!(admin::console)) },
                        "Admin console"
                    )
                )
                breadcrumb_separator()
                breadcrumb_item(breadcrumb_page("Propagation Procedures"))
            )
        )
        <h1>"Propagation Procedures"</h1>
        <ul>
            for p in procedures {
                <li class="flex items-baseline gap-2">
                    <span>
                        <a href=(href!(details, PropagationId(p.id)))>(p.id)</a>
                    </span>
                    <span>
                        <span>(&p.name)</span>
                        <span class="inline-block mx-2">
                            <a
                                class=(button_variants(
                                    ButtonVariant::Secondary,
                                    ButtonSize::default(),
                                ))
                                href=(href!(modify, PropagationId(p.id)))
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
                    "Create a new Propagation Procedure"
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
    let id = *path_param::<PropagationId>(cx)?;

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

    Ok(see_other(href!(details, PropagationId(id)).resolve(cx)))
}

#[route(POST "/admin/propagation/new")]
pub async fn do_create(
    cx: &Cx,
    Form(params): Form<PropagationProcedureParams>,
) -> topcoat::Result<SeeOther> {
    let _user = require_user_with_permission(cx, PermissionCode::PropagationEdit).await?;

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
        href!(details, PropagationId(proc.id)).resolve(cx),
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

#[page("/admin/propagation/{propagation_id}")]
pub async fn modify(cx: &Cx) -> topcoat::Result<impl View> {
    let _user = require_user_with_permission(cx, PermissionCode::PropagationEdit).await?;
    let id = *path_param::<PropagationId>(cx)?;
    let propagation = PropagationProcedure::get_by_id(&mut db(cx), id).await?;
    Ok(view! {
        <hgroup>
            breadcrumb(
                breadcrumb_list(
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! { href=(href!(admin::console)) },
                            "Admin console"
                        )
                    )
                    breadcrumb_separator()
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! { href=(href!(list)) },
                            "Propagation Procedures"
                        )
                    )
                    breadcrumb_separator()
                    breadcrumb_page((id))
                )
            )
            <h1>"Edit a Propagation Procedure"</h1>
        </hgroup>
        propagation_procedure_form(
            procedure: Some(&propagation),
            attrs: attributes! {
                method="POST"
                action=(href!(do_modify, PropagationId(id)))
            },
            button("Update")
        )
    })
}

#[page("/admin/propagation/new")]
pub async fn new(cx: &Cx) -> topcoat::Result<impl View> {
    let _user = require_user_with_permission(cx, PermissionCode::PropagationCreate).await?;
    Ok(view! {
        <hgroup>
            breadcrumb(
                breadcrumb_list(
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! { href=(href!(admin::console)) },
                            "Admin console"
                        )
                    )
                    breadcrumb_separator()
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! { href=(href!(list)) },
                            "Propagation Procedures"
                        )
                    )
                    breadcrumb_separator()
                    breadcrumb_page("New")
                )
            )
            <h1>"Create a New Propagation Procedure"</h1>
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
    let propagation_id = path_param::<PropagationId>(cx)?;
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
    trace!(?procedure);
    Ok(view! {
        <div class="flex flex-col gap-6">
            <section>
                <h1>(&procedure.name)</h1>
                <section class="flex flex-col gap-6">
                    <div>
                        item_term("Type")
                        item_value((procedure.r#type.to_string()))
                    </div>
                    <div>
                        item_term("Notes")
                        item_value((procedure.notes.as_deref().unwrap_or_default()))
                    </div>
                    <div>
                        item_term("Instructions")
                        item_value((&procedure.instructions))
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
                </section>
            </section>
            <section>
                card(
                    card_header(card_title("Taxa"))
                    card_content(
                        if !taxa.is_empty() {
                            <ul>
                                for taxon in taxa {
                                    <li class="flex items-baseline gap-2">
                                        <span>(taxon.id)</span>
                                        <span class="latin">
                                            <a href=(href!(taxa::details, taxa::TaxonId(taxon.id)))>
                                                (&taxon.complete_name)
                                            </a>
                                        </span>
                                    </li>
                                }
                            </ul>
                        } else {
                            "None"
                        }
                    )
                )
            </section>
        </div>
    })
}
