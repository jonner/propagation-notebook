use libpropagation::auth::{Session, UserEmail, UserProfile};
use serde::Deserialize;
use topcoat::{
    context::{Cx, try_context},
    icon::icon,
    metadata::Metadata,
    router::{
        Body, Method, StatusCode,
        content::Form,
        error::{RouterErrorExt, SeeOther, internal_server_error, rewrite, see_other},
        href, page, route,
    },
    runtime::{Event, procedure, signal},
    view::{Attributes, View, attributes, component, view},
};

use crate::{
    Subtitle,
    assets::mdi,
    components::{
        avatar::*, button::*, card::*, field::*, input::input, pn::item_term, pn::item_value,
        textarea::textarea,
    },
    context::{db, require_user},
    util::editable_text,
};

pub(crate) mod admin;

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
async fn email_address(cx: &Cx, address: String, confirmed: bool) -> topcoat::Result<impl View> {
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

#[procedure("/api/user/display-name")]
pub(crate) async fn set_display_name(cx: &Cx, name: String) -> topcoat::Result<()> {
    tracing::debug!(?name);
    let user = require_user(cx).await?;
    UserProfile::upsert_by_user_id(user.id)
        .name(name)
        .exec(&mut db(cx))
        .await?;
    Ok(())
}

#[component]
pub(crate) async fn display_name(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user(cx).await?;
    let profiles = UserProfile::filter_by_user_id(user.id)
        .exec(&mut db(cx))
        .await?;
    let editable = signal(cx, || false);
    Ok(view! {
        let dname = profiles.first().and_then(|profile| profile.name.as_ref());
        editable_text(
            val: dname,
            editable: editable.clone(),
            attrs: attributes! {
                id="name-input"
                name="name"
                @change=$(async |e: Event| {
                    set_display_name(e.target.value).await;
                    editable.set(false);
                })
            }
        )
    })
}

#[procedure("/api/user/description")]
pub(crate) async fn set_description(cx: &Cx, value: String) -> topcoat::Result<()> {
    let user = require_user(cx).await?;
    UserProfile::upsert_by_user_id(user.id)
        .description(value)
        .exec(&mut db(cx))
        .await?;
    Ok(())
}

#[component]
pub(crate) async fn description(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user(cx).await?;
    let profiles = UserProfile::filter_by_user_id(user.id)
        .exec(&mut db(cx))
        .await?;
    let editable = signal(cx, || false);

    Ok(view! {
        let desc = profiles.first().and_then(|profile| profile.description.as_ref());
        editable_text(
            val: desc,
            editable: editable.clone(),
            multiline: true,
            attrs: attributes! {
                id="description-input"
                name="description"
                @change=$(async |e: Event| {
                    set_description(e.target.value).await;
                    editable.set(false);
                })
            }
        )
    })
}

#[page("/user/account", with(Metadata::new().subtitle("User Account")))]
pub(crate) async fn account(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user(cx).await?;
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
                        attrs: attributes! { class="flex flex-col gap-6" },
                        <div>
                            item_term("Display Name")
                            item_value(display_name())
                        </div>
                        <div>
                            item_term("Description")
                            item_value(description())
                        </div>
                        <div>
                            item_term("Picture")
                            item_value(
                                avatar(
                                    attrs: attributes! { class="inline-block" },
                                    avatar_fallback(icon(data: mdi::ACCOUNT))
                                )
                            )
                        </div>
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
                        attrs: attributes! { class="flex flex-col gap-6" },
                        <div>
                            item_term("Username")
                            item_value((&user.username))
                        </div>
                        <div>
                            item_term("Password")
                            item_value(<a href=(href!(change_password))>"Change"</a>)
                        </div>
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
                        attrs: attributes! { class="flex flex-col gap-6" },
                        <div>
                            item_term("Email Addresses")
                            item_value(
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
                        </div>
                    )
                )
            </section>
        </div>
    })
}

#[component]
pub async fn user_profile_form(
    cx: &Cx,
    profile: Option<&UserProfile>,
    #[default] attrs: Attributes,
) -> topcoat::Result<impl View> {
    Ok(view! {
        <form (attrs)>
            field_set(
                field(
                    field_label(attrs: attributes! { for="name-input" }, "Display Name")
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

#[page("/user/profile", with(Metadata::new().subtitle("Edit User Profile")))]
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

#[derive(Debug, thiserror::Error)]
#[error("Failed to rotate the session")]
struct SessionRotationError;

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
    let mut db = db(cx);
    let mut txn = db.transaction().await?;
    if !errors.has_error() {
        match user.change_password(&mut txn, &params.new_password).await {
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
    if let Some(rotation) = topcoat::session::rotate(cx).await? {
        // remove all existing sessions
        Session::delete_by_user_id(&mut txn, user.id).await?;
        let timestamp: jiff::Timestamp = rotation.session.expires_at.try_into()?;
        // remember the new hash
        Session::create()
            .token_hash(Vec::from(&*rotation.session.token_hash))
            .expires_at(timestamp)
            .user_id(user.id)
            .exec(&mut txn)
            .await?;
    } else {
        return Err(internal_server_error(SessionRotationError).into());
    }
    txn.commit().await?;

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

#[page("/user/password", with(Metadata::new().subtitle("Change password")))]
pub(crate) async fn change_password(cx: &Cx) -> topcoat::Result<impl View> {
    let _user = require_user(cx).await?;
    let rejected = try_context::<ChangePasswordErrors>(cx);
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

#[cfg(test)]
mod tests {
    use libpropagation::auth::{Session, User, hash_password};
    use topcoat::{
        context::BaseUrl,
        cookie::{CookieLayer, Key},
        router::{
            Router, RouterBuilderDiscoverExt, header, request::Request as HttpRequest,
            response::Response as HttpResponse,
        },
        runtime::RuntimeLayer,
        session::Token,
    };

    use super::*;

    const OLD_PASSWORD: &str = "correct horse battery staple";
    const NEW_PASSWORD: &str = "Tr0ub4dor&3 parakeet";

    /// A web application backed by an in-memory SQLite database.
    ///
    /// The tests deliberately stick to routes that never render a page (login,
    /// password change, profile update): those answer with redirects or 401s,
    /// so the harness does not need an asset bundle for the shell layout.
    struct TestApp {
        router: Router,
        db: toasty::Db,
    }

    impl TestApp {
        async fn start() -> Self {
            let db = libpropagation::connect("sqlite::memory:", true)
                .await
                .expect("test database should connect");
            let router = Router::builder()
                .discover()
                .with(db.clone())
                .with(Key::generate())
                .with(BaseUrl::new("http://localhost"))
                .layer(RuntimeLayer::new())
                .layer(CookieLayer::new())
                .build();
            Self { router, db }
        }

        async fn create_user(&self, username: &str, password: &str) -> User {
            User::create()
                .username(username.to_string())
                .pwhash(hash_password(password).expect("password should hash"))
                .exec(&mut self.db.clone())
                .await
                .expect("user should be created")
        }

        async fn post_form(
            &self,
            uri: &str,
            cookie: Option<&str>,
            form: &[(&str, &str)],
        ) -> HttpResponse {
            let body = serde_urlencoded::to_string(form).expect("form should encode");
            let mut request = HttpRequest::builder()
                .method(Method::POST)
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
            if let Some(cookie) = cookie {
                request = request.header(header::COOKIE, cookie);
            }
            self.router
                .handle(request.body(Body::from(body)).expect("request should build"))
                .await
        }

        async fn login(&self, username: &str, password: &str) -> String {
            let response = self
                .post_form(
                    "/auth/login",
                    None,
                    &[("username", username), ("password", password)],
                )
                .await;
            assert_eq!(
                response.status(),
                StatusCode::SEE_OTHER,
                "login as {username} should succeed"
            );
            session_cookie(&response).expect("login should issue a session cookie")
        }

        /// Whether `cookie` still authenticates, without rendering a page.
        async fn auth_status(&self, cookie: &str) -> StatusCode {
            self.post_form("/user/profile", Some(cookie), &[])
                .await
                .status()
        }

        async fn sessions(&self) -> Vec<Session> {
            Session::all()
                .exec(&mut self.db.clone())
                .await
                .expect("sessions should load")
        }

        async fn password_works(&self, username: &str, password: &str) -> bool {
            let user = User::get_by_username(&mut self.db.clone(), username)
                .await
                .expect("user should exist");
            user.verify_password(password).is_ok()
        }
    }

    /// Returns the `name=value` part of the session cookie set by `response`.
    fn session_cookie(response: &HttpResponse) -> Option<String> {
        response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .find_map(|value| {
                let pair = value.split(';').next()?.trim();
                let (name, token) = pair.split_once('=')?;
                (name.ends_with("session") && !token.is_empty()).then(|| pair.to_string())
            })
    }

    fn cookie_token(cookie: &str) -> Token {
        let parsed = topcoat::cookie::Cookie::parse_encoded(cookie.to_string())
            .expect("session cookie should parse");
        Token::decode(parsed.value_trimmed()).unwrap_or_else(|error| {
            panic!("session cookie {cookie:?} should hold a token: {error}")
        })
    }

    async fn change_password(
        app: &TestApp,
        cookie: &str,
        current: &str,
        new: &str,
    ) -> HttpResponse {
        app.post_form(
            "/user/password",
            Some(cookie),
            &[
                ("current_password", current),
                ("new_password", new),
                ("new_password_confirm", new),
            ],
        )
        .await
    }

    #[tokio::test]
    async fn changing_password_revokes_other_sessions_and_rotates_the_current_one() {
        let app = TestApp::start().await;
        let user = app.create_user("alice", OLD_PASSWORD).await;
        let current = app.login("alice", OLD_PASSWORD).await;
        let other_device = app.login("alice", OLD_PASSWORD).await;
        assert_ne!(current, other_device);
        assert_eq!(app.sessions().await.len(), 2);

        let response = change_password(&app, &current, OLD_PASSWORD, NEW_PASSWORD).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // the current device keeps working under a fresh token
        let rotated =
            session_cookie(&response).expect("password change should rotate the session cookie");
        assert_ne!(rotated, current);

        // every other session is gone, and the rotated one is stored
        let sessions = app.sessions().await;
        assert_eq!(sessions.len(), 1, "all other sessions should be revoked");
        assert_eq!(sessions[0].user_id, user.id);
        assert_eq!(sessions[0].token_hash, Vec::from(*cookie_token(&rotated).hash()));

        // the old tokens no longer authenticate, the rotated one does
        assert_eq!(app.auth_status(&current).await, StatusCode::UNAUTHORIZED);
        assert_eq!(
            app.auth_status(&other_device).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(app.auth_status(&rotated).await, StatusCode::SEE_OTHER);

        // the stored password changed: the old one no longer verifies
        assert!(app.password_works("alice", NEW_PASSWORD).await);
        assert!(!app.password_works("alice", OLD_PASSWORD).await);
    }

    #[tokio::test]
    async fn password_change_requires_authentication() {
        let app = TestApp::start().await;
        app.create_user("carol", OLD_PASSWORD).await;

        let response = change_password(&app, "", OLD_PASSWORD, NEW_PASSWORD).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(app.sessions().await.is_empty());
        assert!(
            app.password_works("carol", OLD_PASSWORD).await,
            "the password should be unchanged"
        );
    }
}
