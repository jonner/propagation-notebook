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
    components::{button::button, field::*, input::input, label::label, textarea::textarea},
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
        <form class="flex flex-col gap-6" method="POST" action=(href!(update_profile))>
            input(
                attrs: attributes! { type="hidden" name="userid" value=(user.id.to_string()) }
            )
            field_set(
                field(
                    field_label(attrs: attributes! { for="name-input" }, "Name")
                    input(
                        attrs: attributes! {
                            id="name-input"
                            type="text"
                            name="name"
                            value=(profile.as_ref().map(|p| p.name.as_ref()))
                        }
                    )
                )
                field(
                    field_label(
                        attrs: attributes! { for="description-input" },
                        "Description"
                    )
                    field_description(
                            "Write a short description about yourself"

                    )
                    textarea(
                        attrs: attributes! {
                            id="description-input"
                            name="description"
                        },
                        (profile.as_ref().map(|p| p.description.as_ref()))
                    )
                )
            )
            button(attrs: attributes! { type="submit" }, "Update")
        </form>
    })
}
