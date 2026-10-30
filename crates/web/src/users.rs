use libpropagation::auth::{UserEmail, UserProfile};
use serde::Deserialize;
use topcoat::{
    context::{Cx, try_request_context},
    icon::icon,
    router::{
        Body, Method, StatusCode,
        content::Form,
        error::{RouterErrorExt, SeeOther, rewrite, see_other},
        href, page, route,
    },
    view::{Attributes, Child, View, attributes, class, component, view},
};

use crate::{
    components::{avatar::*, button::button, card::*, field::*, input::input, textarea::textarea},
    context::{db, require_user},
    mdi,
};

#[derive(Deserialize, Debug)]
pub struct UpdateProfileParams {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[route(POST "/user/profile")]
pub(crate) async fn update_profile(
    cx: &Cx,
    Form(params): Form<UpdateProfileParams>,
) -> topcoat::Result<SeeOther> {
    let user = require_user(cx).await.ok_or_unauthorized()?;
    UserProfile::upsert_by_user_id(user.id)
        .name(params.name)
        .description(params.description)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(href!(account).resolve(cx)))
}

#[component]
async fn edit_icon(#[default] mut attrs: Attributes) -> topcoat::Result<impl View> {
    Ok(view! {
        <a class=(class! { "mx-2", attrs.remove("class") }) (attrs)>
            icon(
                data: mdi::EDIT,
                label: "Edit",
                attrs: attributes! { class="inline-block" }
            )
        </a>
    })
}
#[component]
async fn setting_row(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div class=(class! { "py-3", attrs.remove("class") }) (attrs)>(child)</div>
    })
}

#[component]
async fn setting_name(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div class=(class! { "font-bold text-lg", attrs.remove("class") }) (attrs)>
            (child)
        </div>
    })
}

#[component]
async fn setting_value(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <div
            class=(class! { "text-muted-foreground px-2", attrs.remove("class") })
            (attrs)
        >
            (child)
        </div>
    })
}

#[component]
async fn email_address(address: String, confirmed: bool) -> topcoat::Result<impl View> {
    Ok(view! {
        (address)
        if confirmed {
            icon(
                data: mdi::CHECK_BOLD,
                attrs: attributes! { class="inline-block text-leaf" },
                label: "Confirmed"
            )
        }
    })
}

#[page("/user/account")]
pub(crate) async fn account(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user(cx).await?;
    let prof = UserProfile::get_by_user_id(&mut db(cx), user.id)
        .await
        .map(Some)
        .or_else(|e| {
            if e.is_record_not_found() {
                Ok(None)
            } else {
                Err(e)
            }
        })?;
    let emails = UserEmail::filter_by_user_id(user.id)
        .exec(&mut db(cx))
        .await?;

    Ok(view! {
        <h1>"User Account"</h1>
        <div class="flex flex-col gap-6">
            <section>
                card(
                    card_header(
                        card_title("Personal Information")
                        card_description("Your personal profile on this website")
                    )
                    card_content(
                        setting_row(
                            setting_name("Display Name")
                            setting_value(
                                if let Some(s) = prof
                                    .as_ref()
                                    .and_then(|p| p.name.as_ref()) {
                                    (&s)
                                } else {
                                    <span class="italic">"None"</span>
                                }
                                edit_icon(
                                    attrs: attributes! { href=(href!(profile_edit)) }
                                )
                            )
                        )
                        setting_row(
                            setting_name("Description")
                            setting_value(
                                if let Some(s) = prof
                                    .as_ref()
                                    .and_then(|p| p.description.as_ref()) {
                                    (&s)
                                } else {
                                    <span class="italic">"None"</span>
                                }
                                edit_icon(
                                    attrs: attributes! { href=(href!(profile_edit)) }
                                )
                            )
                        )
                        setting_row(
                            setting_name("Picture")
                            setting_value(
                                avatar(
                                    attrs: attributes! { class="inline-block" },
                                    avatar_fallback(icon(data: mdi::ACCOUNT))
                                )
                            )
                        )
                    )
                )
            </section>
            <section>
                card(
                    card_header(
                        card_title("Account Security")
                        card_description(
                            "Information about how you log into the website"
                        )
                    )
                    card_content(
                        setting_row(
                            setting_name("Username")
                            setting_value((&user.username))
                        )
                        setting_row(
                            setting_name("Password")
                            setting_value(<a href=(href!(change_password))>"Change"</a>)
                        )
                    )
                )
            </section>
            <section>
                card(
                    card_header(
                        card_title("Contact Information")
                        card_description("Ways to contact you")
                    )
                    card_content(
                        setting_row(
                            setting_name("Email Addresses")
                            setting_value(
                                if !emails.is_empty() {
                                    for email in emails {
                                        email_address(
                                            address: email.address,
                                            confirmed: email.confirmed
                                        )
                                    }
                                } else {
                                    <span class="italic">"None"</span>
                                }
                            )
                        )
                    )
                )
            </section>
        </div>
    })
}

#[component]
pub async fn user_profile_form(
    profile: Option<&UserProfile>,
    #[default] attrs: Attributes,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <form (attrs)>
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
            button(attrs: attributes! { type="submit" }, "Update Profile")
        </form>
    })
}

#[page("/user/profile")]
pub(crate) async fn profile_edit(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user(cx).await?;
    let prof = UserProfile::get_by_user_id(&mut db(cx), user.id)
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
        <h1>"User Profile"</h1>
        user_profile_form(
            attrs: attributes! {
                class="flex flex-col gap-6"
                method="POST"
                action=(href!(update_profile))
            },
            profile: prof.as_ref()
        )
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

    Ok(see_other(href!(account).resolve(cx)))
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
    let _user = require_user(cx).await?;
    let rejected = try_request_context::<ChangePasswordErrors>(cx);
    let errors = rejected.cloned().unwrap_or_default();

    Ok(view! {
        if rejected.is_some() {
            (StatusCode::UNPROCESSABLE_ENTITY)
        }

        <h1>"Change your password"</h1>
        card(
            attrs: attributes! { class="mx-9 md:mx-auto md:max-w-1/2" },
            card_content(
                <form
                    class="flex flex-col gap-9"
                    method="POST"
                    action=(href!(do_change_password))
                >
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
            )
        )
    })
}
