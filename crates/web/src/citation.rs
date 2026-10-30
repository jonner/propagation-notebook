use libpropagation::{auth::PermissionCode, citation::Citation};
use serde::Deserialize;
use topcoat::{
    context::{Cx, try_request_context},
    router::{
        Body, Method, StatusCode,
        content::Form,
        error::{RouterErrorExt, SeeOther, rewrite, see_other},
        href, page, path_param, route,
    },
    runtime::signal,
    view::{Attributes, Child, View, attributes, class, component, view},
};
use tracing::debug;
use url::Url;

use crate::{
    actions::{Action, ActionItemType, permitted_actions},
    components::{
        action_menu::*, alert_dialog::*, breadcrumb::*, button::*, dialog::*, field::*,
        input::input,
    },
    context::{current_user, db, require_user_with_permission},
    flash::{self, FlashMessage},
};

path_param!(pub citation_id: u64, error = bad_request);

#[page("/citations/{citation_id}")]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let mut db = db(cx);
    let id = *path_param::<CitationId>(cx)?;
    let citation = Citation::get_by_id(&mut db, id).await.ok_or_not_found()?;
    let delete_dialog_open = signal(cx, || false);

    let actions = [
        Action {
            name: "Edit".to_string(),
            permission: Some(PermissionCode::CitationEdit),
            item_type: ActionItemType::Link,
            attrs: attributes! { href=(href!(modify, CitationId(id))) },
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
        <hgroup>
            breadcrumb(
                breadcrumb_list(
                    breadcrumb_item(breadcrumb_page("Citations"))
                    breadcrumb_separator()
                    breadcrumb_item(breadcrumb_page((id)))
                )
            )
            <h1 class="flex items-center">
                "Citation Details"
                let actions = permitted_actions(current_user(cx).await, &actions);
                if !actions.is_empty() {
                    <div class="ms-auto">action_menu(actions: actions.as_slice())</div>
                }
            </h1>
        </hgroup>
        <dt>"Title"</dt>
        <dd>(citation.title)</dd>
        <dt>"Author"</dt>
        <dd>(citation.author)</dd>
        <dt>"Publication Year"</dt>
        <dd>(citation.publication_year)</dd>
        <dt>"Url"</dt>
        <dd>
            if let Some(url) = citation.url {
                <a href=(&url)>(&url)</a>
            }
        </dd>
        <dt>"Container Title"</dt>
        <dd>(citation.container_title)</dd>
        <dt>"doi"</dt>
        <dd>
            if let Some(doi) = citation.doi {
                <a href=(format!("https://doi.org/{}", doi))>(doi)</a>
            }
        </dd>
        <dt>"Access Date"</dt>
        <dd>
            if let Some(date) = citation.access_date {
                (date.to_string())
            }
        </dd>
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
                    <form action=(href!(delete_citation, CitationId(id))) method="POST">
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
    let id = *path_param::<CitationId>(cx)?;
    match Citation::delete_by_id(&mut db(cx), id).await {
        Ok(_) => {
            flash::push_message(cx, FlashMessage::Success(format!("Deleted citation {id}")))?;
            Ok(see_other(href!(crate::home).resolve(cx)))
        }
        Err(e) => {
            flash::push_message(
                cx,
                FlashMessage::Failure(format!("Failed to delete citation: {e}")),
            )?;
            Err(see_other(href!(details, CitationId(id)).resolve(cx)).into())
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
        href!(details, CitationId(citation.id)).resolve(cx),
    ))
}

#[component]
pub async fn required() -> topcoat::Result<impl View> {
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
    let rejected = try_request_context::<CitationRejected>(cx);
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

#[page("/citations/new")]
pub async fn create(cx: &Cx) -> topcoat::Result<impl View> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationCreate)
            .await?;
    Ok(view! {
        <hgroup>
            breadcrumb(
                breadcrumb_list(
                    breadcrumb_item(breadcrumb_page("Citations"))
                    breadcrumb_separator()
                    breadcrumb_item(breadcrumb_page("Create"))
                )
            )
            <h1>"Create a new citation"</h1>
        </hgroup>
        citation_form(
            attrs: attributes! { method="POST" action=(href!(do_create)) },
            button("Create")
        )
    })
}

#[page("/admin/citations/{citation_id}/edit")]
pub async fn modify(cx: &Cx) -> topcoat::Result<impl View> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationEdit)
            .await?;
    let id = path_param::<CitationId>(cx)?;
    let citation = Citation::get_by_id(&mut db(cx), id).await?;
    Ok(view! {
        <hgroup>
            breadcrumb(
                breadcrumb_list(
                    breadcrumb_item(breadcrumb_page("Citations"))
                    breadcrumb_separator()
                    breadcrumb_link(
                        attrs: attributes! { href=(href!(details, CitationId(*id))) },
                        (id)
                    )
                    breadcrumb_separator()
                    breadcrumb_item(breadcrumb_page("Edit"))
                )
            )
            <h1>"Edit a Citation"</h1>
        </hgroup>
        citation_form(
            citation: Some(&citation),
            attrs: attributes! { method="POST" action=(href!(do_modify, CitationId(*id))) },
            button("Update")
        )
    })
}

#[route(POST "/admin/citations/{citation_id}")]
pub async fn do_modify(cx: &Cx, Form(params): Form<CitationParams>) -> topcoat::Result<SeeOther> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationEdit)
            .await?;

    let id = *path_param::<CitationId>(cx)?;
    if let Some(errors) = params.validate() {
        return Err(
            rewrite(href!(modify, CitationId(id)).resolve(cx), Body::empty())
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
    Ok(see_other(href!(details, CitationId(id)).resolve(cx)))
}

#[page("/admin/citations")]
pub async fn admin(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user_with_permission(cx, PermissionCode::CitationEdit).await?;
    let citations = Citation::all().exec(&mut db(cx)).await?;
    Ok(view! {
        <h2>"Edit Citations"</h2>
        <ul class="flex flex-col gap-3">
            for citation in citations {
                <li class="flex items-baseline gap-2">
                    <span>(citation.id)</span>
                    <span>
                        <a href=(href!(modify, CitationId(citation.id)))>
                            (citation.format_cse())
                        </a>
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
                        "Create a new citation source"
                    </a>
                </li>
            }
        </ul>
    })
}
