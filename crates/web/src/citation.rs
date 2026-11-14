use std::collections::HashMap;

use libpropagation::{auth::PermissionCode, citation::Citation, taxonomy::Taxon};
use serde::Deserialize;
use topcoat::{
    context::{Cx, memoize, try_context},
    icon::icon,
    metadata::Metadata,
    router::{
        Body, Method, Slot, StatusCode,
        content::Form,
        error::{RouterErrorExt, SeeOther, rewrite, see_other},
        href, layout, page, path_param, route,
    },
    runtime::signal,
    view::{Attributes, Child, View, attributes, class, component, view},
};
use tracing::debug;
use url::Url;

use crate::{
    Subtitle,
    actions::{Action, ActionItemType, permitted_actions},
    assets::mdi,
    components::{
        action_menu::*,
        alert_dialog::*,
        breadcrumb::*,
        button::*,
        card::*,
        dialog::*,
        field::*,
        hover_card::*,
        input::input,
        pn::{item_term, item_value},
    },
    context::{current_user, db, require_user_with_permission},
    crumb::{Crumb, crumbs},
    flash::{self, FlashMessage},
    propagation, taxa,
};

path_param!(pub citation_id: u64 else bad_request);

#[memoize(as_ref)]
async fn citation(cx: &Cx) -> topcoat::Result<Citation> {
    let id = *citation_id(cx)?;
    Ok(Citation::filter_by_id(id)
        .include(Citation::fields().cleaning_procedures().cleaning().taxon())
        .include(Citation::fields().propagation_procedures().propagation())
        .include(Citation::fields().taxon_notes().note().taxon())
        .include(
            Citation::fields()
                .taxon_propagation_procedures()
                .taxon_propagation()
                .taxon(),
        )
        .one()
        .exec(&mut db(cx))
        .await
        .ok_or_not_found()?)
}

#[page(
    "/citations/{citation_id}",
    with(Metadata::from_fn(
        async |cx| {
            let citation = citation(cx).await?;
            Ok(Metadata::new().subtitle(&citation.title))
        },
    ))
)]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let citation = citation(cx).await?;
    let delete_dialog_open = signal(cx, || false);

    let mut cited_taxa: HashMap<u64, &Taxon> = Default::default();
    for cite in citation.taxon_notes.get() {
        let note = cite.note.get();
        cited_taxa.insert(note.taxon_id, note.taxon.get());
    }
    for cite in citation.taxon_propagation_procedures.get() {
        cited_taxa.insert(cite.taxon_id, cite.taxon_propagation.get().taxon.get());
    }
    for cite in citation.cleaning_procedures.get() {
        let cleaning = cite.cleaning.get();
        cited_taxa.insert(cleaning.taxon_id, cleaning.taxon.get());
    }
    let cited_procedures = citation
        .propagation_procedures
        .get()
        .iter()
        .map(|pp| pp.propagation.get())
        .collect::<Vec<_>>();

    let actions = [
        Action {
            name: "Edit".to_string(),
            permission: Some(PermissionCode::CitationEdit),
            item_type: ActionItemType::Link,
            attrs: attributes! { href=(href!(modify, citation_id: citation.id)) },
        },
        Action {
            name: "Delete".to_string(),
            permission: Some(PermissionCode::CitationDelete),
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
            <section>
                <hgroup>
                    breadcrumb(
                        breadcrumb_list(
                            breadcrumb_item(breadcrumb_page("Cited Works"))
                            breadcrumb_separator()
                            breadcrumb_item(breadcrumb_page((citation.id)))
                        )
                    )
                    <h1 class="flex items-center">
                        "Cited Work"
                        let actions = permitted_actions(current_user(cx).await, &actions);
                        if !actions.is_empty() {
                            <div class="ms-auto">
                                action_menu(actions: actions.as_slice())
                            </div>
                        }
                    </h1>
                </hgroup>
                card(
                    card_content(
                        <section class="flex flex-col gap-6">
                            <div>
                                item_term("Title")
                                item_value((&citation.title))
                            </div>
                            <div>
                                item_term("Author")
                                item_value((&citation.author))
                            </div>
                            <div>
                                item_term("Publication Year")
                                item_value((citation.publication_year))
                            </div>
                            <div>
                                item_term("Url")
                                item_value(
                                    if let Some(url) = citation.url.as_ref() {
                                        <a href=(&url)>(&url)</a>
                                    } else {
                                        <span class="text-muted-foreground">"Not specified"</span>
                                    }
                                )
                            </div>
                            <div>
                                item_term("Container Title")
                                item_value((citation.container_title.as_ref()))
                            </div>
                            <div>
                                item_term("doi")
                                item_value(
                                    if let Some(doi) = citation.doi.as_ref() {
                                        <a href=(format!("https://doi.org/{}", doi))>(doi)</a>
                                    }
                                )
                            </div>
                            <div>
                                item_term("Access Date")
                                item_value(
                                    if let Some(date) = citation.access_date {
                                        (date.to_string())
                                    }
                                )
                            </div>
                        </section>
                    )
                )
            </section>
            <section class="flex flex-col gap-6">
                <h2>"Citations"</h2>
                <div class="empty:hidden px-3">
                    if !cited_taxa.is_empty() {
                        <div>
                            <h3>"Taxa"</h3>
                            <ul>
                                for (id, taxon) in cited_taxa {
                                    <li>
                                        <a href=(href!(taxa::details, taxa::taxon_id: id))>
                                            (&taxon.complete_name)
                                        </a>
                                    </li>
                                }
                            </ul>
                        </div>
                    }
                    if !cited_procedures.is_empty() {
                        <div>
                            <h3>"Propagation Procedures"</h3>
                            <ul>
                                for procedure in cited_procedures {
                                    <li>
                                        <a
                                            href=(href!(
                                                propagation::details,
                                                propagation::propagation_id: procedure.id,
                                            ))
                                        >
                                            (&procedure.name)
                                        </a>
                                    </li>
                                }
                            </ul>
                        </div>
                    }
                </div>
            </section>
        </div>
        alert_dialog(
            open: delete_dialog_open.get(),
            dialog_content(
                dialog_header(
                    dialog_title("Delete this citation?")
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
                        action=(href!(delete_citation, citation_id: citation.id))
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

#[route(POST "/admin/citations/delete/{citation_id}")]
pub async fn delete_citation(cx: &Cx) -> topcoat::Result<SeeOther> {
    let _ = require_user_with_permission(cx, PermissionCode::CitationDelete).await?;
    let id = *citation_id(cx)?;
    match Citation::delete_by_id(&mut db(cx), id).await {
        Ok(_) => {
            flash::push_message(
                cx,
                FlashMessage::Success(format!("Deleted cited work {id}")),
            )?;
            Ok(see_other(href!(crate::home).resolve(cx)))
        }
        Err(e) => {
            flash::push_message(
                cx,
                FlashMessage::Failure(format!("Failed to delete cited work: {e}")),
            )?;
            Err(see_other(href!(details, citation_id: id).resolve(cx)).into())
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
struct CitationParams {
    pub title: String,
    pub url: Option<String>,
    pub author: String,
    pub access_date: Option<jiff::civil::Date>,
    pub publication_year: Option<i16>,
    pub container_title: Option<String>,
    pub doi: Option<String>,
}

impl CitationParams {
    fn validate(&self) -> Option<CitationParamErrors> {
        let mut errors = CitationParamErrors::default();
        if self.title.trim().is_empty() {
            errors.title = Some("Title cannot be empty".to_string())
        }
        if self.author.trim().is_empty() {
            errors.author = Some("Author cannot be empty".to_string())
        }
        if let Some(url_str) = self.url.as_ref()
            && let Err(e) = Url::parse(url_str)
        {
            errors.url = Some(e.to_string())
        }
        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }
}

impl From<&Citation> for CitationParams {
    fn from(value: &Citation) -> Self {
        Self {
            title: value.title.clone(),
            url: value.url.clone(),
            author: value.author.clone(),
            access_date: value.access_date,
            publication_year: value.publication_year,
            container_title: value.container_title.clone(),
            doi: value.doi.clone(),
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
struct CitationParamErrors {
    pub title: Option<String>,
    pub url: Option<String>,
    pub author: Option<String>,
    pub access_date: Option<String>,
    pub publication_year: Option<String>,
    pub container_title: Option<String>,
    pub doi: Option<String>,
}

impl CitationParamErrors {
    fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.url.is_none()
            && self.author.is_none()
            && self.access_date.is_none()
            && self.publication_year.is_none()
            && self.container_title.is_none()
            && self.doi.is_none()
    }
}

#[derive(Deserialize, Debug)]
struct CitationRejected {
    form: CitationParams,
    errors: CitationParamErrors,
}

#[route(POST "/admin/citations/new")]
pub async fn do_create(cx: &Cx, Form(params): Form<CitationParams>) -> topcoat::Result<SeeOther> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationCreate)
            .await?;

    if let Some(errors) = params.validate() {
        debug!("Failed parameter validation");
        return Err(rewrite(href!(create).resolve(cx), Body::empty())
            .method(Method::GET)
            .with(CitationRejected {
                form: params,
                errors,
            })
            .into());
    };

    let citation = Citation::create()
        .title(params.title)
        .author(params.author)
        .url(params.url)
        .access_date(params.access_date)
        .publication_year(params.publication_year)
        .container_title(params.container_title)
        .doi(params.doi)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(
        href!(details, citation_id: citation.id).resolve(cx),
    ))
}

#[component]
pub async fn required(cx: &Cx) -> topcoat::Result<impl View> {
    Ok(view! { <span class="text-destructive">"*"</span> })
}

#[component]
pub async fn citation_form(
    cx: &Cx,
    #[default] citation: Option<&Citation>,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    let title = signal(cx, || citation.map(|c| c.title.clone()).unwrap_or_default());
    let author = signal(cx, || {
        citation.map(|c| c.author.clone()).unwrap_or_default()
    });
    let rejected = try_context::<CitationRejected>(cx);
    let errors = rejected.map(|r| r.errors.clone()).unwrap_or_default();
    let values = rejected
        .map(|r| r.form.clone())
        .or_else(|| citation.map(|c| c.into()))
        .unwrap_or_default();
    Ok(view! {
        if rejected.is_some() {
            (StatusCode::UNPROCESSABLE_ENTITY)
        }
        <form class=(class!("flex flex-col gap-6", attrs.remove("class"))) (attrs)>
            field_set(
                field_group(
                    field(
                        field_label(
                            attrs: attributes! { for="citation-title" },
                            "Title"
                            required()
                        )
                        field_description(
                            "The title of the work being cited (e.g. article, web page, book, etc.)"
                        )
                        input(
                            attrs: attributes! {
                                id="citation-title"
                                type="text"
                                name="title"
                                value=(values.title)
                                @input=$(|e: topcoat::runtime::Event| title.set(
                                        e.target.value,
                                    ))
                            }
                        )
                        if let Some(error) = errors.title {
                            field_error((error))
                        }
                    )
                    field(
                        field_label(
                            attrs: attributes! { for="citation-author" },
                            "Author"
                            required()
                        )
                        field_description("The author of the work being cited")
                        input(
                            attrs: attributes! {
                                id="citation-author"
                                type="text"
                                name="author"
                                value=(values.author)
                                @input=$(|e: topcoat::runtime::Event| author.set(
                                        e.target.value,
                                    ))
                            }
                        )
                        if let Some(error) = errors.author {
                            field_error((error))
                        }
                    )
                    field(
                        field_label(
                            attrs: attributes! { for="citation-container" },
                            "Containing work"
                        )
                        field_description(
                            "The name of the containing object (e.g. journal, project, website, etc.)"
                        )
                        input(
                            attrs: attributes! {
                                id="citation-container"
                                type="text"
                                name="container_title"
                                value=(values.container_title)
                            }
                        )
                    )
                    field(
                        field_label(
                            attrs: attributes! { for="citation-publication" },
                            "Publication Year"
                        )
                        field_description(
                            "The year that the work being cited was published"
                        )
                        input(
                            attrs: attributes! {
                                id="citation-publication"
                                type="text"
                                name="publication_year"
                                value=(values.publication_year)
                            }
                        )
                    )
                    field(
                        field_label(attrs: attributes! { for="citation-url" }, "URL")
                        field_description(
                            "An optional web address for the work being cited"
                        )
                        input(
                            attrs: attributes! {
                                id="citation-url"
                                type="text"
                                name="url"
                                value=(values.url)
                            }
                        )
                    )
                    field(
                        field_label(
                            attrs: attributes! { for="citation-doi" },
                            "Digital Object Identifier (doi)"
                        )
                        field_description("An optional doi for the work being cited")
                        input(
                            attrs: attributes! {
                                id="citation-doi"
                                type="text"
                                name="doi"
                                value=(values.doi)
                            }
                        )
                    )
                    field(
                        field_label(
                            attrs: attributes! { for="citation-accessdate" },
                            "Access Date"
                        )
                        field_description("The date that the work was last accessed")
                        input(
                            attrs: attributes! {
                                id="citation-accessdate"
                                type="text"
                                name="access_date"
                                value=(values.access_date.map(|d| d.to_string()))
                            }
                        )
                    )
                    (child)
                )
            )
        </form>
    })
}

#[layout(
    "/admin/citations",
    with(Crumb::new(
        async |cx| Ok(("Cited Works".to_string(), Some(href!(admin).resolve(cx)))),
    ))
)]
pub async fn citation_admin_shell(cx: &Cx, slot: Slot<'_>) -> topcoat::Result<impl View> {
    Ok(view! { (slot) })
}

#[page(
    "/admin/citations/new",
    with(Metadata::new().subtitle("New cited work")),
    with(Crumb::new(async |_cx| Ok(("New".to_string(), None))))
)]
pub async fn create(cx: &Cx) -> topcoat::Result<impl View> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationCreate)
            .await?;
    Ok(view! {
        <hgroup>
            crumbs()
            <h1>"Create a new citation"</h1>
        </hgroup>
        citation_form(
            attrs: attributes! { method="POST" action=(href!(do_create)) },
            button("Create")
        )
    })
}

#[page(
    "/admin/citations/{citation_id}/edit",
    with(Metadata::from_fn(
        async |cx| {
            let citation = citation(cx).await?;
            Ok(
                Metadata::new().subtitle(
                    format!("{} | Administration Console", citation.title),
                ),
            )
        },
    )),
    with(Crumb::new(
        async |cx| {
            let citation = citation(cx).await?;
            Ok((citation.id.to_string(), None))
        },
    ))
)]
pub async fn modify(cx: &Cx) -> topcoat::Result<impl View> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationEdit)
            .await?;
    let id = *citation_id(cx)?;
    let citation = Citation::get_by_id(&mut db(cx), id).await?;
    Ok(view! {
        <hgroup>
            crumbs()
            <h1>"Edit a Cited Work"</h1>
        </hgroup>
        citation_form(
            citation: Some(&citation),
            attrs: attributes! { method="POST" action=(href!(do_modify, citation_id: id)) },
            button("Update")
        )
    })
}

#[route(POST "/admin/citations/{citation_id}")]
pub async fn do_modify(cx: &Cx, Form(params): Form<CitationParams>) -> topcoat::Result<SeeOther> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationEdit)
            .await?;

    let id = *citation_id(cx)?;
    if let Some(errors) = params.validate() {
        return Err(
            rewrite(href!(modify, citation_id: id).resolve(cx), Body::empty())
                .method(Method::GET)
                .with(CitationRejected {
                    form: params.clone(),
                    errors,
                })
                .into(),
        );
    };

    Citation::update_by_id(id)
        .title(params.title)
        .author(params.author)
        .url(params.url)
        .access_date(params.access_date)
        .publication_year(params.publication_year)
        .container_title(params.container_title)
        .doi(params.doi)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(href!(details, citation_id: id).resolve(cx)))
}

#[page("/admin/citations", with(Metadata::new().subtitle("Cited Works")))]
pub async fn admin(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user_with_permission(cx, PermissionCode::CitationEdit).await?;
    let citations = Citation::all().exec(&mut db(cx)).await?;
    Ok(view! {
        crumbs()
        <h2>"Cited Works"</h2>
        <ul class="flex flex-col gap-3">
            for citation in citations {
                <li class="flex items-baseline gap-2">
                    <span>
                        <a href=(href!(details, citation_id: citation.id))>
                            (citation.id)
                        </a>
                    </span>
                    <span>
                        <span>(citation.format_cse())</span>
                        <span class="inline-block mx-2">
                            <a
                                class=(button_variants(
                                    ButtonVariant::Secondary,
                                    ButtonSize::default(),
                                ))
                                href=(href!(modify, citation_id: citation.id))
                            >
                                icon(data: mdi::EDIT)
                            </a>
                        </span>
                    </span>
                </li>
            }
            if user.has_permission(PermissionCode::CitationCreate) {
                <li>
                    <a
                        href=(href!(create))
                        class=(button_variants(
                            ButtonVariant::Primary,
                            ButtonSize::default(),
                        ))
                    >
                        "Create a new cited work"
                    </a>
                </li>
            }
        </ul>
    })
}

#[component]
pub async fn citation_card(cx: &Cx, citation: &Citation) -> topcoat::Result<impl View> {
    let primary_link = citation.url.clone().or(citation
        .doi
        .as_ref()
        .map(|doi| format!("https://doi.org/{doi}")));
    Ok(view! {
        <div><a href=(primary_link)>(&citation.title)</a></div>
        <div>
            (&citation.author)
            if let Some(container) = citation.container_title.as_ref() {
                icon(attrs: attributes! { class="inline-flex" }, data: mdi::DOT)
                <em>(&container)</em>
            }
            if let Some(year) = citation.publication_year {
                icon(attrs: attributes! { class="inline-flex" }, data: mdi::DOT)
                (year.to_string())
            }
        </div>
        if citation.doi.is_some() || citation.access_date.is_some() {
            <div class="flex gap-3">
                if let Some(doi) = citation.doi.as_ref() {
                    <span>
                        "DOI: "
                        <a href=(format!("https://doi.org/{doi}"))>(doi)</a>
                    </span>
                }
                if let Some(date) = citation.access_date {
                    <span>(format!("(Accessed {})", date))</span>
                }
            </div>
        }
    })
}

#[component]
pub async fn citation_reference(cx: &Cx, citation: &Citation) -> topcoat::Result<impl View> {
    Ok(view! {
        <span class="align-super text-xs ps-1">
            "["
            hover_card(
                <a href=(href!(details, citation_id: citation.id))>(citation.id)</a>
                hover_card_content(citation_card(citation: citation))
            )
            "]"
        </span>
    })
}
