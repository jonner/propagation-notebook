use libpropagation::auth::{PermissionCode, Session, User};
use topcoat::{
    context::{Cx, app_context, memoize},
    router::error::{RouterErrorExt, UnauthorizedError, unauthorized},
    session::{self, TokenHash},
};

pub fn db(cx: &Cx) -> toasty::Db {
    app_context::<toasty::Db>(cx).clone()
}

pub async fn current_user(cx: &Cx) -> Option<&User> {
    if let Ok(Some(user)) = load_user(cx).await {
        Some(user)
    } else {
        None
    }
}

pub async fn session_hash(cx: &Cx) -> Option<TokenHash> {
    session::token_hash(cx).await.ok().flatten()
}

pub async fn persist_session(cx: &Cx, user: &User) -> topcoat::Result<()> {
    let session = session::start(cx).await?;
    let timestamp: jiff::Timestamp = session.expires_at.try_into()?;
    let _ = Session::create()
        .token_hash(Vec::from(*session.token_hash))
        .user(user)
        .expires_at(timestamp);
    Ok(())
}

pub async fn delete_session(cx: &Cx) -> topcoat::Result<()> {
    if let Some(hash) = session::stop(cx).await? {
        Session::delete_by_token_hash(&mut db(cx), Vec::from(*hash)).await?;
    }
    Ok(())
}

#[memoize(as_ref)]
async fn load_user(cx: &Cx) -> topcoat::Result<Option<User>> {
    let mut db = db(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Ok(None);
    };

    let session = Session::filter_by_token_hash(Vec::from(*hash))
        .include(
            Session::fields()
                .user()
                .user_roles()
                .role()
                .role_permissions()
                .permission(),
        )
        .one()
        .exec(&mut db)
        .await?;
    Ok(Some(session.user.into_inner()))
}

async fn require_user(cx: &Cx) -> topcoat::Result<&User, UnauthorizedError> {
    load_user(cx)
        .await
        .ok_or_unauthorized()
        .and_then(|val| val.as_ref().ok_or_unauthorized())
}

async fn require_user_with_permission(
    cx: &Cx,
    permission: PermissionCode,
) -> topcoat::Result<&User, UnauthorizedError> {
    let user = require_user(cx).await?;
    if user
        .user_roles
        .get()
        .iter()
        .find(|user_role| {
            user_role
                .role
                .get()
                .role_permissions
                .get()
                .iter()
                .find(|rp| rp.permission.get().code == permission)
                .is_some()
        })
        .is_some()
    {
        Ok(user)
    } else {
        Err(unauthorized())
    }
}
