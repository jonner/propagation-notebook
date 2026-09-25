use libpropagation::auth::UserProfile;
use serde::Deserialize;
use topcoat::{
    context::Cx,
    router::{
        content::Form,
        error::{RouterErrorExt, SeeOther, forbidden, see_other},
        header::REFERER,
        href, page,
        request::headers,
        route,
    },
    view::{View, attributes, view},
};
use uuid::Uuid;

use crate::{
    components::{button::button, input::input, label::label, textarea::textarea},
    context::{db, require_user},
};

#[derive(Deserialize, Debug)]
struct UpdateProfileParams {
    userid: Uuid,
    name: Option<String>,
    description: Option<String>,
}

#[route(POST "/user/profile")]
pub(crate) async fn update_profile(
    cx: &Cx,
    Form(params): Form<UpdateProfileParams>,
) -> topcoat::Result<SeeOther> {
    let user = require_user(cx).await.ok_or_unauthorized()?;
    if params.userid != user.id {
        return Err(forbidden().into());
    }
    UserProfile::upsert_by_user_id(params.userid)
        .name(params.name)
        .description(params.description)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(
        headers(cx)
            .get(REFERER)
            .and_then(|value| value.to_str().ok().map(|v| v.to_string()))
            .unwrap_or_else(|| href!(profile).resolve(cx)),
    ))
}

#[page("/user/profile")]
pub(crate) async fn profile(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user(cx).await?;
    let profile = UserProfile::get_by_user_id(&mut db(cx), user.id)
        .await
        .map(Some)
        .or_else(|e| {
            if e.is_record_not_found() {
                Ok(None)
            } else {
                Err(e)
            }
        })?;

    Ok(view! {
        <h1>(&user.username)</h1>
        <form class="flex flex-col gap-3" method="POST" action=(href!(update_profile))>
            <div class="flex flex-col gap-1">
                input(
                    attrs: attributes! { type="hidden" name="userid" value=(user.id.to_string()) }
                )
                label(attrs: attributes! { class="text-muted-foreground" }, "Name:")
                input(
                    attrs: attributes! {
                        type="text"
                        placeholder="Name"
                        name="name"
                        value=(profile.as_ref().map(|p| p.name.as_ref()))
                    }
                )
            </div>
            <div class="flex flex-col gap-1">
                label(
                    attrs: attributes! { class="text-muted-foreground" },
                    "Description:"
                )
                textarea(
                    attrs: attributes! {
                        placeholder="Write a short description about yourself"
                        name="description"
                    },
                    (profile.as_ref().map(|p| p.description.as_ref()))
                )
            </div>
            button(attrs: attributes! { type="submit" }, "Update")
        </form>
    })
}
