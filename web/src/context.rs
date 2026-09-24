use jiff::SignedDuration;
use libpropagation::auth::{PermissionCode, Session, User};
use topcoat::{
    context::{Cx, app_context, memoize},
    router::error::{RouterErrorExt, UnauthorizedError, forbidden},
    session,
};

pub fn db(cx: &Cx) -> toasty::Db {
    app_context::<toasty::Db>(cx).clone()
}

pub async fn current_user(cx: &Cx) -> Option<&User> {
    if let Ok(Some(session)) = load_session(cx).await {
        Some(session.user.get())
    } else {
        None
    }
}

pub async fn persist_session(cx: &Cx, user: &mut User) -> topcoat::Result<()> {
    let session = session::start(cx).await?;
    let timestamp: jiff::Timestamp = session.expires_at.try_into()?;
    user.login(&mut db(cx), &*session.token_hash, timestamp)
        .await?;
    Ok(())
}

#[memoize(as_ref)]
pub async fn load_session(cx: &Cx) -> topcoat::Result<Option<Session>> {
    let hash = session::token_hash(cx).await?;
    if let Some(hash) = hash {
        let db_session = Session::filter_by_token_hash(Vec::from(*hash))
            .include(Session::fields().user().roles())
            .include(Session::fields().user().permissions())
            .one()
            .exec(&mut db(cx))
            .await?;
        // refresh the session if it's near expiration
        if (jiff::Timestamp::now().duration_until(db_session.expires_at))
            < SignedDuration::from_hours(7 * 24)
            && let Some(topcoat_session) = session::refresh(cx).await?
        {
            let timestamp: jiff::Timestamp = topcoat_session.expires_at.try_into()?;
            Session::update_by_token_hash(Vec::from(*topcoat_session.token_hash))
                .expires_at(timestamp)
                .exec(&mut db(cx))
                .await?;
        }
        Ok(Some(db_session))
    } else {
        Ok(None)
    }
}

pub async fn delete_session(cx: &Cx) -> topcoat::Result<()> {
    if let Some(hash) = session::stop(cx).await? {
        Session::delete_by_token_hash(&mut db(cx), Vec::from(*hash)).await?;
    }
    Ok(())
}

pub async fn require_user(cx: &Cx) -> topcoat::Result<&User, UnauthorizedError> {
    load_session(cx).await.ok_or_unauthorized().and_then(|val| {
        val.as_ref()
            .map(|session| session.user.get())
            .ok_or_unauthorized()
    })
}

pub async fn require_user_with_permission(
    cx: &Cx,
    permission: PermissionCode,
) -> topcoat::Result<&User> {
    let user = require_user(cx).await?;
    if user.has_permission(permission) {
        Ok(user)
    } else {
        Err(forbidden().into())
    }
}
