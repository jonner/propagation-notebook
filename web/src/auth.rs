use libpropagation::auth::User;
use serde::{Deserialize, Serialize};
use topcoat::{
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, redirect, see_other},
        href, page, query_params, route,
    },
    view::{View, attributes, view},
};

use crate::{
    components::{button::button, card::*, field::*, input::input},
    context::{current_user, db, delete_session, persist_session},
    home,
};

#[route(POST "/auth/logout")]
pub(crate) async fn do_logout(cx: &Cx) -> topcoat::Result<SeeOther> {
    delete_session(cx).await?;
    Ok(see_other(href!(login).resolve(cx)))
}

#[derive(Debug, Deserialize)]
struct LoginParams {
    username: String,
    password: String,
    redirect: Option<String>,
}

#[page("/auth/logout")]
pub(crate) async fn logout(cx: &Cx) -> topcoat::Result<impl View> {
    if current_user(cx).await.is_none() {
        return Err(redirect(href!(login).resolve(cx)).into());
    };

    Ok(view! {
        <div class="w-full m-auto lg:max-w-1/2 flex flex-col items-center">
            <h1>"Log out"</h1>
            <form method="POST" class="w-full flex flex-col gap-6">
                button("Log Out")
            </form>
        </div>
    })
}

#[route(POST "/auth/login")]
pub(crate) async fn do_login(Form(data): Form<LoginParams>, cx: &Cx) -> topcoat::Result<SeeOther> {
    let mut user = User::get_by_username(&mut db(cx), data.username).await?;
    match user.verify_password(&data.password) {
        Ok(()) => {
            persist_session(cx, &mut user).await?;
            Ok(see_other(
                data.redirect.unwrap_or_else(|| href!(home).resolve(cx)),
            ))
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug, Clone, Serialize)]
#[query_params(error = bad_request)]
pub struct LoginFormParams {
    pub redirect: Option<String>,
}

#[page("/auth/login")]
pub(crate) async fn login(cx: &Cx) -> topcoat::Result<impl View> {
    let user = current_user(cx).await;
    let params = query_params::<LoginFormParams>(cx)?;
    if user.is_some() {
        return Err(see_other(href!(crate::home).resolve(cx)).into());
    }

    Ok(view! {
        card(
            attrs: attributes! { class="w-full mx-auto my-12 lg:max-w-1/2" },
            card_header(card_title("Log In"))
            card_content(
                attrs: attributes! { class="flex flex-col items-center" },
                <form method="POST" class="w-full flex flex-col gap-6">
                    field_set(
                        field(
                            field_label(
                                attrs: attributes! { for="username-input" },
                                "Username"
                            )
                            input(
                                attrs: attributes! { id="username-input" name="username" }
                            )
                        )
                        field(
                            field_label(
                                attrs: attributes! { for="password-input" },
                                "Password"
                            )
                            input(
                                attrs: attributes! { type="password" id="password-input" name="password" }
                            )
                        )
                        input(
                            attrs: attributes! {
                                type="hidden"
                                name="redirect"
                                value=(params.redirect.as_ref())
                            }
                        )
                    )
                    button("Log In")
                </form>
            )
        )
    })
}
