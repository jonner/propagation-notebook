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
    runtime::{procedure, signal},
    view::{View, attributes, view},
};

use crate::{
    components::{
        alert_dialog::*, button::*, dialog::*, dropdown_menu::*, input::input, label::label,
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
                            if user.has_permission(PermissionCode::CitationDelete) {
                                dropdown_menu_item("Edit")
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
struct CitationCreateParams {
    pub title: String,
    pub url: Option<String>,
    pub author: String,
    pub access_date: Option<jiff::civil::Date>,
    pub publication_year: Option<i16>,
    pub container_title: Option<String>,
    pub doi: Option<String>,
}

#[route(POST "/citations/create")]
pub async fn do_create(
    cx: &Cx,
    Form(params): Form<CitationCreateParams>,
) -> topcoat::Result<SeeOther> {
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

#[page("/citations/create")]
pub async fn create(cx: &Cx) -> topcoat::Result<impl View> {
    let _user =
        require_user_with_permission(cx, libpropagation::auth::PermissionCode::CitationCreate)
            .await?;
    Ok(view! {
        <h1>"Create a new citation"</h1>
        <form method="POST" action=(href!(do_create)) class="flex flex-col gap-3">
            <div class="flex flex-col gap-1">
                label(
                    "Title:"
                    <span class="text-destructive">"*"</span>
                )
                input(
                    attrs: attributes! {
                        type="text"
                        name="title"
                        placeholder="Enter the title of the work being cited (e.g. article, web page, book, etc.)"
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label(
                    "Author:"
                    <span class="text-destructive">"*"</span>
                )
                input(
                    attrs: attributes! {
                        type="text"
                        name="author"
                        placeholder="Enter the author of the work being cited"
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label("Containing work")
                input(
                    attrs: attributes! {
                        type="text"
                        name="container_title"
                        placeholder="Enter the name of the containing object (e.g. journal, project, website, etc.)"
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label("Publication Year:")
                input(
                    attrs: attributes! {
                        type="text"
                        name="publication_year"
                        placeholder="Enter an optional publication year for the work being cited"
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label("Url:")
                input(
                    attrs: attributes! {
                        type="text"
                        name="url"
                        placeholder="Enter an optional url to the work being cited"
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label("Digital Object Identifier (doi):")
                input(
                    attrs: attributes! {
                        type="text"
                        name="doi"
                        placeholder="Enter an optional doi for the work being cited"
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label("Access Date:")
                input(
                    attrs: attributes! {
                        type="text"
                        name="access_date"
                        value=(&jiff::Zoned::now().date().to_string())
                        placeholder="Enter the date that the work was last accessed"
                    }
                )
            </div>
            button("Create")
        </form>
    })
}
