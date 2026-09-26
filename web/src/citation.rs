use libpropagation::{auth::PermissionCode, citation::Citation};
use serde::Deserialize;
use topcoat::{
    context::Cx,
    icon::icon,
    router::{
        content::Form,
        error::{RouterErrorExt, SeeOther, see_other},
        href, page, path_param, route,
    },
    runtime::{Event, procedure, signal},
    view::{Attributes, Child, View, attributes, class, component, view},
};

use crate::{
    components::{
        alert_dialog::*,
        button::*,
        dialog::*,
        dropdown_menu::*,
        input::input,
        label::label,
        pn::{info_hover_card, required_icon},
    },
    context::{current_user, db, require_user_with_permission},
    mdi,
};

path_param!(pub citation_id: u64, error = bad_request);

#[page("/citations/{citation_id}")]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let user = current_user(cx).await;
    let mut db = db(cx);
    let id = path_param::<CitationId>(cx)?;
    let citation = Citation::get_by_id(&mut db, id).await.ok_or_not_found()?;
    let delete_dialog_open = signal(cx, || false);
    Ok(view! {
        <h1 class="flex items-center">
            "Citation "
            (citation.id)
            if let Some(user) = user {
                let menu_open = signal(cx, || false);
                if user.has_permission(PermissionCode::CitationEdit)
                    || user.has_permission(PermissionCode::CitationDelete) {
                    dropdown_menu(
                        attrs: attributes! { class="ms-auto" :open=$(menu_open.get()) },
                        dropdown_menu_trigger(icon(data: mdi::DOTS_VERTICAL))
                        dropdown_menu_content(
                            alignment: DropdownMenuAlignment::Right,
                            if user.has_permission(PermissionCode::CitationDelete) {
                                dropdown_menu_navigation_item(
                                    attrs: attributes! {
                                        href=(href!(modify, CitationId(*id)))
                                        @click=$(|_e: Event| menu_open.set(false))
                                    },
                                    "Modify"
                                )
                            }
                            if user.has_permission(PermissionCode::CitationDelete) {
                                dropdown_menu_item(
                                    attrs: attributes! {
                                        @click=$(|_e: topcoat::runtime::Event| {
                                            delete_dialog_open.set(true);
                                            menu_open.set(false);
                                        })
                                        class="text-destructive"
                                    },
                                    "Delete"
                                )
                            }
                        )
                    )
                }
            }
        </h1>
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
                    let idstr = citation.id.to_string();
                    button(
                        variant: ButtonVariant::Destructive,
                        attrs: attributes! {
                            @click=$(async |e: topcoat::runtime::Event| {
                                e.prevent_default();
                                delete_dialog_open.set(false);
                                delete_citation(idstr).await;
                                //FIXME: Handle response
                            })
                        },
                        "Delete"
                    )
                )
            )
        )
    })
}

// FIXME: use integer when topcoat 0.11 comes
#[procedure]
pub async fn delete_citation(cx: &Cx, id: String) -> topcoat::Result<()> {
    let _ = require_user_with_permission(cx, PermissionCode::CitationDelete).await?;
    let id = id.parse::<u64>()?;
    Citation::delete_by_id(&mut db(cx), id).await?;
    Ok(())
}

#[derive(Deserialize, Debug)]
struct CitationParams {
    pub title: String,
    pub url: Option<String>,
    pub author: String,
    pub access_date: Option<jiff::civil::Date>,
    pub publication_year: Option<i16>,
    pub container_title: Option<String>,
    pub doi: Option<String>,
}

#[route(POST "/citations/create")]
pub async fn do_create(cx: &Cx, Form(params): Form<CitationParams>) -> topcoat::Result<SeeOther> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationCreate)
            .await?;
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
pub async fn citation_form(
    cx: &Cx,
    #[default] citation: Option<&Citation>,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <form class=(class!("flex flex-col gap-6", attrs.remove("class"))) (attrs)>
            <div class="flex flex-col gap-2">
                label(
                    "Title"
                    required_icon()
                    info_hover_card(
                        "Enter the title of the work being cited (e.g. article, web page, book, etc.)"
                    )
                )
                input(
                    attrs: attributes! {
                        type="text"
                        name="title"
                        value=(citation.map(|c| &c.title))
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label(
                    "Author"
                    required_icon()
                    info_hover_card("The author of the work being cited")
                )
                input(
                    attrs: attributes! {
                        type="text"
                        name="author"
                        value=(citation.map(|c| &c.author))
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label(
                    "Containing work"
                    info_hover_card(
                        "The name of the containing object (e.g. journal, project, website, etc.)"
                    )
                )
                input(
                    attrs: attributes! {
                        type="text"
                        name="container_title"
                        value=(citation.and_then(|c| c.container_title.as_ref()))
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label(
                    "Publication Year"
                    info_hover_card(
                        "The year that the work being cited was published"
                    )
                )
                input(
                    attrs: attributes! {
                        type="text"
                        name="publication_year"
                        value=(citation.and_then(|c| c.publication_year))
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label(
                    "URL"
                    info_hover_card("An optional web address for the work being cited")
                )
                input(
                    attrs: attributes! {
                        type="text"
                        name="url"
                        value=(citation.and_then(|c| c.url.as_ref()))
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label(
                    "Digital Object Identifier (doi)"
                    info_hover_card("An optional doi for the work being cited")
                )
                input(
                    attrs: attributes! {
                        type="text"
                        name="doi"
                        value=(citation.and_then(|c| c.doi.as_ref()))
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label(
                    "Access Date"
                    info_hover_card("The date that the work was last accessed")
                )
                input(
                    attrs: attributes! {
                        type="text"
                        name="access_date"
                        value=(citation.and_then(
                            |c| c.access_date.map(|d| d.to_string()),
                        ))
                    }
                )
            </div>
            (child)
        </form>
    })
}

#[page("/citations/create")]
pub async fn create(cx: &Cx) -> topcoat::Result<impl View> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationCreate)
            .await?;
    Ok(view! {
        <h1>"Create a new citation"</h1>
        citation_form(
            attrs: attributes! { method="POST" action=(href!(do_create)) },
            button("Create")
        )
    })
}

#[page("/citations/{citation_id}/modify")]
pub async fn modify(cx: &Cx) -> topcoat::Result<impl View> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationEdit)
            .await?;
    let id = path_param::<CitationId>(cx)?;
    let citation = Citation::get_by_id(&mut db(cx), id).await?;
    Ok(view! {
        <h1>"Modify a citation"</h1>
        citation_form(
            citation: Some(&citation),
            attrs: attributes! { method="POST" action=(href!(do_modify, CitationId(*id))) },
            button("Update")
        )
    })
}

#[route(POST "/citations/{citation_id}")]
pub async fn do_modify(cx: &Cx, Form(params): Form<CitationParams>) -> topcoat::Result<SeeOther> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationEdit)
            .await?;
    let id = path_param::<CitationId>(cx)?;
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
    Ok(see_other(href!(details, CitationId(*id)).resolve(cx)))
}
