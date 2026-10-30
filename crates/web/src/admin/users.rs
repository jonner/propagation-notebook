use std::collections::HashSet;

use libpropagation::auth::{PermissionCode, Role, User, UserProfile, UserRole};
use topcoat::{
    context::Cx,
    router::{
        content::Form,
        error::{RouterErrorExt, SeeOther, bad_request, see_other},
        href, page, path_param, route,
    },
    view::{Attributes, View, attributes, component, view},
};
use uuid::Uuid;

use crate::{
    components::{
        breadcrumb::*,
        button::button,
        card::*,
        checkbox::*,
        label::*,
        pn::{item_term, item_value},
        tooltip::*,
    },
    context::{db, require_user_with_permission},
    users::{UpdateProfileParams, user_profile_form},
};

path_param!(user_id: Uuid, error = bad_request);

#[page("/admin/users")]
pub async fn list(cx: &Cx) -> topcoat::Result<impl View> {
    _ = require_user_with_permission(cx, PermissionCode::UserList).await?;
    let users = User::all().exec(&mut db(cx)).await?;

    Ok(view! {
        breadcrumb(
            breadcrumb_list(
                breadcrumb_item(breadcrumb_page("Admin console"))
                breadcrumb_separator()
                breadcrumb_item(breadcrumb_page("Users"))
            )
        )
        <h2>"User List"</h2>
        <ul>
            for user in users {
                <li><a href=(href!(edit, UserId(user.id)))>(user.username)</a></li>
            }
        </ul>
    })
}

#[page("/admin/users/{user_id}")]
pub async fn edit(cx: &Cx) -> topcoat::Result<impl View> {
    let user = require_user_with_permission(cx, PermissionCode::UserEdit).await?;
    let user_id = *path_param::<UserId>(cx)?;
    let user_to_edit = User::filter_by_id(user_id)
        .include(User::fields().profile())
        .include(User::fields().roles())
        .one()
        .exec(&mut db(cx))
        .await?;

    Ok(view! {
        <h2>"Edit User"</h2>
        <section class="flex flex-col gap-6">
            card(
                card_header(card_title("User Info"))
                card_content(
                    attrs: attributes! { class="flex flex-col gap-6" },
                    <div>
                        item_term("User ID")
                        item_value((user_id.to_string()))
                    </div>
                    <div>
                        item_term("Username")
                        item_value((&user_to_edit.username))
                    </div>
                )
            )
            card(
                card_header(card_title("User Profile"))
                card_content(
                    user_profile_form(
                        attrs: attributes! {
                            class="flex flex-col gap-4"
                            method="POST"
                            action=(href!(update_profile, UserId(user_id)))
                        },
                        profile: user_to_edit.profile.get().as_ref()
                    )
                )
            )
            if user.has_permission(PermissionCode::UserRoles) {
                card(
                    card_header(card_title("Roles"))
                    card_content(
                        user_roles_form(
                            attrs: attributes! {
                                method="POST"
                                action=(href!(update_roles, UserId(user_id)))
                            },
                            user: &user_to_edit
                        )
                    )
                )
            }
        </section>
    })
}

#[component]
async fn user_roles_form(
    cx: &Cx,
    #[default] attrs: Attributes,
    user: &User,
) -> topcoat::Result<impl View> {
    let all_roles = Role::all()
        .include(Role::fields().permissions())
        .exec(&mut db(cx))
        .await?;
    Ok(view! {
        <form (attrs)>
            <div class="flex flex-col gap-3">
                for role in all_roles {
                    let id = role.id.to_string();
                    let checked = user.roles.get().iter().any(|r| r.id == role.id);
                    <div class="flex items-center gap-2">
                        checkbox(
                            attrs: attributes! { id=(&id) name="role" value=(&id) checked=(checked) }
                        )
                        tooltip(
                            label(attrs: attributes! { for=(&id) }, (&role.name))
                            tooltip_content(
                                if role.permissions.get().is_empty() {
                                    "No Permissions"
                                } else {
                                    <ul>
                                        for permission in role.permissions.get() {
                                            <li>(permission.code.to_string())</li>
                                        }
                                    </ul>
                                }
                            )
                        )
                    </div>
                }
                button(attrs: attributes! { type="submit" }, "Update Roles")
            </div>
        </form>
    })
}

#[route(POST "/admin/users/{user_id}/roles")]
pub async fn update_roles(
    cx: &Cx,
    Form(params): Form<Vec<(String, String)>>,
) -> topcoat::Result<SeeOther> {
    _ = require_user_with_permission(cx, PermissionCode::UserRoles).await?;
    if params.iter().any(|(name, _)| name != "role") {
        return Err(bad_request("invalid form field").into());
    }
    let params = params
        .into_iter()
        .map(|(_, value)| Uuid::parse_str(&value))
        .collect::<Result<Vec<Uuid>, _>>()
        .ok_or_bad_request("Could not parse form value as UUID")?;
    let mut db = db(cx);
    let user_id = *path_param::<UserId>(cx)?;
    let user_to_edit = User::filter_by_id(user_id)
        .include(User::fields().roles())
        .one()
        .exec(&mut db)
        .await?;
    let new_set = params.iter().collect::<HashSet<_>>();
    let old_set = user_to_edit
        .roles
        .get()
        .iter()
        .map(|r| r.id)
        .collect::<HashSet<_>>();
    let roles_to_add = params
        .iter()
        .filter(|r| !old_set.contains(r))
        .collect::<Vec<_>>();
    let roles_to_remove = user_to_edit
        .roles
        .get()
        .iter()
        .map(|r| r.id)
        .filter(|r| !new_set.contains(r))
        .collect::<Vec<_>>();

    let mut txn = db.transaction().await?;
    let create_queries: Vec<_> = roles_to_add
        .into_iter()
        .map(|r| UserRole::create().user_id(user_to_edit.id).role_id(r))
        .collect();
    toasty::batch(create_queries).exec(&mut txn).await?;
    let remove_queries: Vec<_> = roles_to_remove
        .into_iter()
        .map(|r| UserRole::filter_by_user_id_and_role_id(user_id, r).delete())
        .collect();
    toasty::batch(remove_queries).exec(&mut txn).await?;
    txn.commit().await?;

    Ok(see_other(href!(edit, UserId(user_id)).resolve(cx)))
}

#[route(POST "/admin/users/{user_id}")]
pub async fn update_profile(
    cx: &Cx,
    Form(params): Form<UpdateProfileParams>,
) -> topcoat::Result<SeeOther> {
    _ = require_user_with_permission(cx, PermissionCode::UserEdit).await?;
    let user_id = *path_param::<UserId>(cx)?;
    UserProfile::upsert_by_user_id(user_id)
        .name(params.name)
        .description(params.description)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(href!(edit, UserId(user_id)).resolve(cx)))
}
