use libpropagation::citation::Citation;
use serde::Deserialize;
use topcoat::{
    context::Cx,
    router::{
        content::Form,
        error::{RouterErrorExt, SeeOther, see_other},
        href, page, path_param, route,
    },
    view::{View, attributes, view},
};

use crate::{
    components::{button::button, input::input, label::label},
    context::{db, require_user_with_permission},
};
path_param!(pub citation_id: u64, error = bad_request);

#[page("/citations/{citation_id}")]
pub async fn details(cx: &Cx) -> topcoat::Result<impl View> {
    let mut db = db(cx);
    let id = path_param::<CitationId>(cx)?;
    let citation = Citation::get_by_id(&mut db, id).await.ok_or_not_found()?;
    Ok(view! {
        <h1>
            "Citation "
            (citation.id)
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
    })
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
