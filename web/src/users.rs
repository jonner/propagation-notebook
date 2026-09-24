use topcoat::{
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::context::require_user;

#[page("/user/profile")]
pub(crate) async fn profile(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user(cx).await?;

    Ok(view! {
        <h1>"User Profile"</h1>
        <p>(&user.username)</p>
    })
}
