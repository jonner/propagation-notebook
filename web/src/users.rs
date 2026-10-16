use libpropagation::auth::{User, UserProfile};
use serde::Deserialize;
use topcoat::{
    context::{Cx, try_request_context},
    router::{
        Body, Method, StatusCode,
        content::Form,
        error::{RouterErrorExt, SeeOther, forbidden, rewrite, see_other},
        header::REFERER,
        href, page,
        request::headers,
        route,
    },
    view::{View, attributes, view},
};
use uuid::Uuid;

use crate::{
    components::{button::button, field::*, input::input, textarea::textarea},
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
                    field_description("Write a short description about yourself")
                    textarea(
                        attrs: attributes! { id="description-input" name="description" },
                        (profile.as_ref().map(|p| p.description.as_ref()))
                    )
                )
            )
            button(attrs: attributes! { type="submit" }, "Update")
        </form>
    })
}

#[derive(Deserialize, Debug, Default, Clone)]
struct ChangePasswordParams {
    current_password: String,
    new_password: String,
    new_password_confirm: String,
}

#[route(POST "/user/password")]
pub(crate) async fn do_change_password(
    cx: &Cx,
    Form(params): Form<ChangePasswordParams>,
) -> topcoat::Result<SeeOther> {
    let mut user = require_user(cx).await?.clone();
    let mut errors = ChangePasswordErrors::default();
    if user.verify_password(&params.current_password).is_err() {
        errors.current_password = Some("Password is invalid".to_string());
    };
    if params.new_password != params.new_password_confirm {
        errors.new_password_confirm = Some("Passwords do not match".to_string());
    }
    if !errors.has_error() {
        match user
            .change_password(&mut db(cx), &params.new_password)
            .await
        {
            Ok(()) => (),
            Err(e) => match e {
                libpropagation::auth::ChangePasswordError::InvalidPassword(msg, suggestions) => {
                    errors.new_password = Some((msg, suggestions));
                }
                e => return Err(e.into()),
            },
        };
    }
    if errors.has_error() {
        return Err(rewrite(href!(change_password).resolve(cx), Body::empty())
            .method(Method::GET)
            .with(errors)
            .into());
    }

    Ok(see_other(href!(profile).resolve(cx)))
}

#[derive(Debug, Default, Clone)]
struct ChangePasswordErrors {
    current_password: Option<String>,
    new_password: Option<(String, Vec<String>)>,
    new_password_confirm: Option<String>,
}

impl ChangePasswordErrors {
    fn has_error(&self) -> bool {
        self.current_password.is_some()
            || self.new_password.is_some()
            || self.new_password_confirm.is_some()
    }
}

#[page("/user/password")]
pub(crate) async fn change_password(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user(cx).await?;
    let rejected = try_request_context::<ChangePasswordErrors>(cx);
    let errors = rejected.cloned().unwrap_or_default();

    Ok(view! {
        if rejected.is_some() {
            (StatusCode::UNPROCESSABLE_ENTITY)
        }

        <h1>(&user.username)</h1>
        <h2>"Change your password"</h2>
        <form
            class="flex flex-col gap-6"
            method="POST"
            action=(href!(do_change_password))
        >
            input(
                attrs: attributes! { type="hidden" name="userid" value=(user.id.to_string()) }
            )
            field_set(
                field(
                    field_label(
                        attrs: attributes! { for="current-input" },
                        "Current password"
                    )
                    input(
                        attrs: attributes! {
                            id="current-input"
                            type="password"
                            name="current_password"
                        }
                    )
                    if let Some(msg) = errors.current_password {
                        field_error((msg))
                    }
                )
                field(
                    field_label(
                        attrs: attributes! { for="new-password-1-input" },
                        "New password"
                    )
                    field_description("Enter a new password")
                    input(
                        attrs: attributes! {
                            type="password"
                            id="new-password-1-input"
                            name="new_password"
                        }
                    )
                    if let Some((msg, suggestions)) = errors.new_password {
                        field_error(
                            (msg)
                            if !suggestions.is_empty() {
                                <ul>
                                    for suggestion in suggestions {
                                        <li>(suggestion)</li>
                                    }
                                </ul>
                            }
                        )
                    }
                )
                field(
                    field_label(
                        attrs: attributes! { for="new-password-2-input" },
                        "Confirm new password"
                    )
                    field_description("Re-enter your new password")
                    input(
                        attrs: attributes! {
                            type="password"
                            id="new-password-2-input"
                            name="new_password_confirm"
                        }
                    )
                    if let Some(msg) = errors.new_password_confirm {
                        field_error((msg))
                    }
                )
            )
            button(attrs: attributes! { type="submit" }, "Update")
        </form>
    })
}
