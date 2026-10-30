use libpropagation::auth::PermissionCode;
use topcoat::{
    context::Cx,
    router::{Slot, href, layout, page},
    view::{View, attributes, view},
};

use crate::{citation, components::navigation_menu::*, context::require_user_with_permission};

mod users;

#[layout("/admin")]
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
                            "Citations"
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
                    )
                )
            )
        )
        (slot)
    })
}

#[page("/admin")]
pub async fn console() -> topcoat::Result<impl View> {
    Ok(view! {
        <h1>"Administration Console"</h1>
        <section class="flex flex-col gap-2">
            "Please select a page from the navigation menu above (for now)."
        </section>
    })
}
