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
    components::{button::button, input::input},
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

    Ok(view! {
        if user.is_some() {
            <h1>"Already Logged In"</h1>
        } else {
            <div class="w-full m-auto lg:max-w-1/2 flex flex-col items-center">
                <h1>"Login"</h1>
                <form method="POST" class="w-full flex flex-col gap-6">
                    <div>
                        input(
                            attrs: attributes! { id="username" placeholder="Username" name="username" }
                        )
                    </div>
                    <div>
                        input(
                            attrs: attributes! {
                                type="password"
                                id="password"
                                placeholder="Password"
                                name="password"
                            }
                        )
                    </div>
                    input(
                        attrs: attributes! {
                            type="hidden"
                            name="redirect"
                            value=(params.redirect.as_ref())
                        }
                    )
                    button("Log In")
                </form>
            </div>
        }
    })
}
