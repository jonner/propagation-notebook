use libpropagation::auth::{Permission, PermissionCode};
use serde::Deserialize;
use topcoat::{
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, see_other},
        href, page, path_param, route,
    },
    view::{View, attributes, view},
};
use uuid::Uuid;

use crate::{
    admin,
    components::{breadcrumb::*, button::button, card::*, field::*, input::*},
    context::{db, require_user_with_permission},
};

path_param!(permission_id: Uuid, error = bad_request);

#[page("/admin/permissions")]
pub async fn list(cx: &Cx) -> topcoat::Result<impl View> {
    _ = require_user_with_permission(cx, PermissionCode::PermissionEdit).await?;
    let permissions = Permission::all().exec(&mut db(cx)).await?;

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
                breadcrumb_item(breadcrumb_page("Permissions"))
            )
        )
        <h2>"Permission List"</h2>
        <section class="flex flex-col gap-2">
            <ul class="contents">
                for permission in permissions {
                    <li>
                        <a href=(href!(edit, PermissionId(permission.id)))>
                            (permission.code.to_string())
                        </a>
                    </li>
                }
            </ul>
            <form method="POST" action=(href!(populate_permissions))>
                button(attrs: attributes! { type="submit" }, "Sync Permissions")
            </form>
        </section>
    })
}

#[route(POST "/adming/permissions/populate")]
async fn populate_permissions(cx: &Cx) -> topcoat::Result<SeeOther> {
    _ = Permission::populate_database(&mut db(cx)).await?;
    Ok(see_other(href!(list).resolve(cx)))
}

#[page("/admin/permissions/{permission_id}")]
pub async fn edit(cx: &Cx) -> topcoat::Result<impl View> {
    _ = require_user_with_permission(cx, PermissionCode::PermissionEdit).await?;
    let permission_id = *path_param::<PermissionId>(cx)?;
    let permission = Permission::filter_by_id(permission_id)
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
                    breadcrumb_link(
                        attrs: attributes! { href=(href!(list)) },
                        "Permissions"
                    )
                )
                breadcrumb_separator()
                breadcrumb_item(breadcrumb_page((permission_id.to_string())))
            )
        )
        <h2>"Edit Role"</h2>
        <section class="flex flex-col gap-6">
            card(
                card_header(card_title("Permission Info"))
                card_content(
                    <form
                        class="flex flex-col gap-6"
                        method="POST"
                        action=(href!(update, PermissionId(permission_id)))
                    >
                        field_set(
                            field(
                                field_label("Id")
                                (permission.id.to_string())
                            )
                            field(
                                field_label("Code")
                                (permission.code.to_string())
                            )
                            field(
                                field_label(
                                    attrs: attributes! { for="permission-description-input" },
                                    "Description"
                                )
                                input(
                                    attrs: attributes! {
                                        type="text"
                                        name="description"
                                        value=(&permission.description)
                                        id="permission-description-input"
                                    }
                                )
                            )
                        )
                        button(attrs: attributes! { type="submit" }, "Update")
                    </form>
                )
            )
        </section>
    })
}

#[derive(Debug, Deserialize)]
struct PermissionUpdateParams {
    description: String,
}

#[route(POST "/admin/permissions/{permission_id}")]
pub async fn update(
    cx: &Cx,
    Form(params): Form<PermissionUpdateParams>,
) -> topcoat::Result<SeeOther> {
    _ = require_user_with_permission(cx, PermissionCode::RoleEdit).await?;
    let permission_id = *path_param::<PermissionId>(cx)?;
    Permission::update_by_id(permission_id)
        .description(params.description)
        .exec(&mut db(cx))
        .await?;
    Ok(see_other(
        href!(edit, PermissionId(permission_id)).resolve(cx),
    ))
}
