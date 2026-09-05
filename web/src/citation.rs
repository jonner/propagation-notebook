use libpropagation::citation::Citation;
use topcoat::{
    context::Cx,
    router::{error::RouterErrorExt, page, path_param},
    view::view,
};

use crate::util::db;
path_param!(pub citation_id: u64, error = bad_request);

#[page("/citation/{citation_id}")]
pub async fn details(cx: &Cx) -> topcoat::Result {
    let mut db = db(cx);
    let id = path_param::<CitationId>(cx)?;
    let citation = Citation::get_by_id(&mut db, id).await.ok_or_not_found()?;
    view! {
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
    }
}
