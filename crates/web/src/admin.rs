use libpropagation::auth::PermissionCode;
use topcoat::{
    context::Cx,
    metadata::Metadata,
    router::{Slot, href, layout, page},
    view::{View, attributes, view},
};

use crate::{
    Subtitle, citation,
    components::navigation_menu::*,
    context::require_user_with_permission,
    crumb::{Crumb, crumbs},
};

mod permissions;
mod roles;
mod users;

#[layout(
    "/admin",
    with(Crumb::new(
        async |cx| {
            Ok(("Administration Console".into(), Some(href!(console).resolve(cx))))
        },
    ))
)]
pub async fn admin_shell(cx: &Cx, slot: Slot<'_>) -> topcoat::Result<impl View> {
    let _user = require_user_with_permission(cx, PermissionCode::AdminConsole).await?;
    Ok(view! {
        navigation_menu(
            navigation_menu_list(
                navigation_menu_item(
                    navigation_menu_trigger("Database")
                    navigation_menu_content(
                        attrs: attributes! { class="min-w-sm" },
                        navigation_menu_link(
                            attrs: attributes! { href=(href!(citation::admin)) },
                            "Cited Works"
                        )
                    )
                )
                navigation_menu_item(
                    navigation_menu_trigger("Site administration")
                    navigation_menu_content(
                        attrs: attributes! { class="min-w-sm" },
                        navigation_menu_link(
                            attrs: attributes! { href=(href!(users::list)) },
                            "Users"
                        )
                        navigation_menu_link(
                            attrs: attributes! { href=(href!(roles::list)) },
                            "Roles"
                        )
                        navigation_menu_link(
                            attrs: attributes! { href=(href!(permissions::list)) },
                            "Permissions"
                        )
                    )
                )
            )
        )
        (slot)
    })
}

#[page("/admin", with(Metadata::new().subtitle("Administration Console")))]
pub async fn console(cx: &Cx) -> topcoat::Result<impl View> {
    Ok(view! {
        crumbs()
        <h1>"Administration Console"</h1>
        <section class="flex flex-col gap-2">
            "Please select a page from the navigation menu above (for now)."
        </section>
    })
}
