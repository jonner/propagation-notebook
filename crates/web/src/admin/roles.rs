use std::collections::HashSet;

use libpropagation::auth::{Permission, PermissionCode, Role, RolePermission};
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
    admin,
    components::{
        breadcrumb::*, button::button, card::*, checkbox::*, field::*, input::*, label::*,
        tooltip::*,
    },
    context::{db, require_user_with_permission},
};

path_param!(role_id: Uuid, error = bad_request);

#[page("/admin/roles")]
pub async fn list(cx: &Cx) -> topcoat::Result<impl View> {
    _ = require_user_with_permission(cx, PermissionCode::RoleEdit).await?;
    let roles = Role::all().exec(&mut db(cx)).await?;

    Ok(view! {
        breadcrumb(
            breadcrumb_list(
                breadcrumb_item(
                    breadcrumb_link(
                        attrs: attributes! { href=(href!(admin::console)) },
                        "Admin console"
                    )
                )
                breadcrumb_separator()
                breadcrumb_item(breadcrumb_page("Roles"))
            )
        )
        <h2>"Role List"</h2>
        <ul>
            for role in roles {
                <li><a href=(href!(edit, RoleId(role.id)))>(role.name)</a></li>
            }
        </ul>
    })
}

#[page("/admin/roles/{role_id}")]
pub async fn edit(cx: &Cx) -> topcoat::Result<impl View> {
    _ = require_user_with_permission(cx, PermissionCode::RoleEdit).await?;
    let role_id = *path_param::<RoleId>(cx)?;
    let role = Role::filter_by_id(role_id)
        .include(Role::fields().permissions())
        .one()
        .exec(&mut db(cx))
        .await?;

    Ok(view! {
        breadcrumb(
            breadcrumb_list(
                breadcrumb_item(
                    breadcrumb_link(
                        attrs: attributes! { href=(href!(admin::console)) },
                        "Admin console"
                    )
                )
                breadcrumb_separator()
                breadcrumb_item(
                    breadcrumb_link(attrs: attributes! { href=(href!(list)) }, "Roles")
                )
                breadcrumb_separator()
                breadcrumb_item(breadcrumb_page((role_id.to_string())))
            )
        )
        <h2>"Edit Role"</h2>
        <section class="flex flex-col gap-6">
            card(
                card_header(card_title("Role Info"))
                card_content(
                    <form
                        class="flex flex-col gap-6"
                        method="POST"
                        action=(href!(update, RoleId(role_id)))
                    >
                        field_set(
                            field(
                                field_label(
                                    attrs: attributes! { for="role-name-input" },
                                    "Name"
                                )
                                input(
                                    attrs: attributes! {
                                        type="text"
                                        name="name"
                                        value=(&role.name)
                                        id="role-name-input"
                                    }
                                )
                            )
                        )
                        button(attrs: attributes! { type="submit" }, "Update")
                    </form>
                )
            )
            card(
                card_header(card_title("Permissions"))
                card_content(
                    role_permissions_form(
                        attrs: attributes! {
                            method="POST"
                            action=(href!(update_permissions, RoleId(role_id)))
                        },
                        role: &role
                    )
                )
            )
        </section>
    })
}

#[component]
async fn role_permissions_form(
    cx: &Cx,
    #[default] attrs: Attributes,
    role: &Role,
) -> topcoat::Result<impl View> {
    let all_permissions = Permission::all().exec(&mut db(cx)).await?;
    Ok(view! {
        <form (attrs)>
            <div class="flex flex-col gap-3">
                for permission in all_permissions {
                    let id = permission.id.to_string();
                    let checked = role
                        .permissions
                        .get()
                        .iter()
                        .any(|r| r.id == permission.id);
                    <div class="flex items-center gap-2">
                        checkbox(
                            attrs: attributes! {
                                id=(&id)
                                name="permissions"
                                value=(&id)
                                checked=(checked)
                            }
                        )
                        tooltip(
                            label(
                                attrs: attributes! { for=(&id) },
                                (permission.code.to_string())
                            )
                            tooltip_content((&permission.description))
                        )
                    </div>
                }
                button(attrs: attributes! { type="submit" }, "Update Roles")
            </div>
        </form>
    })
}

#[route(POST "/admin/roles/{role_id}/permissions")]
pub async fn update_permissions(
    cx: &Cx,
    Form(params): Form<Vec<(String, String)>>,
) -> topcoat::Result<SeeOther> {
    _ = require_user_with_permission(cx, PermissionCode::RoleEdit).await?;
    if params.iter().any(|(name, _)| name != "permissions") {
        return Err(bad_request("invalid form field").into());
    }
    let params = params
        .into_iter()
        .map(|(_, value)| Uuid::parse_str(&value))
        .collect::<Result<Vec<Uuid>, _>>()
        .ok_or_bad_request("Could not parse form value as UUID")?;
    let mut db = db(cx);
    let role_id = *path_param::<RoleId>(cx)?;
    let role = Role::filter_by_id(role_id)
        .include(Role::fields().permissions())
        .one()
        .exec(&mut db)
        .await?;
    let new_set = params.iter().collect::<HashSet<_>>();
    let old_set = role
        .permissions
        .get()
        .iter()
        .map(|r| r.id)
        .collect::<HashSet<_>>();
    let roles_to_add = params
        .iter()
        .filter(|r| !old_set.contains(r))
        .collect::<Vec<_>>();
    let roles_to_remove = role
        .permissions
        .get()
        .iter()
        .map(|r| r.id)
        .filter(|r| !new_set.contains(r))
        .collect::<Vec<_>>();

    let mut txn = db.transaction().await?;
    let create_queries: Vec<_> = roles_to_add
        .into_iter()
        .map(|r| RolePermission::create().role_id(role.id).permission_id(r))
        .collect();
    toasty::batch(create_queries).exec(&mut txn).await?;
    let remove_queries: Vec<_> = roles_to_remove
        .into_iter()
        .map(|r| RolePermission::filter_by_role_id_and_permission_id(role_id, r).delete())
        .collect();
    toasty::batch(remove_queries).exec(&mut txn).await?;
    txn.commit().await?;

    Ok(see_other(href!(edit, RoleId(role_id)).resolve(cx)))
}

#[route(POST "/admin/roles/{role_id}")]
pub async fn update(cx: &Cx, Form(name): Form<String>) -> topcoat::Result<SeeOther> {
    _ = require_user_with_permission(cx, PermissionCode::RoleEdit).await?;
    let role_id = *path_param::<RoleId>(cx)?;
    Role::update_by_id(role_id)
        .name(name)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(href!(edit, RoleId(role_id)).resolve(cx)))
}
