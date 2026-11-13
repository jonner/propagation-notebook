use topcoat::{
    context::Cx,
    icon::icon,
    view::{Attributes, View, attributes, component, element_id, view},
};

use crate::{
    actions::{Action, ActionItemType, permitted_actions},
    assets::mdi,
    components::{dropdown_menu::*, floating::FloatingAlign},
    context::current_user,
};

#[component]
pub async fn action_menu(
    cx: &Cx,
    #[default] attrs: Attributes,
    actions: &[Action],
) -> topcoat::Result<impl View> {
    let menu_id = element_id(cx);
    let permitted = permitted_actions(current_user(cx).await, actions);

    Ok(view! {
        if !permitted.is_empty() {
            dropdown_menu(
                attrs: attrs,
                dropdown_menu_trigger(
                    attrs: attributes! {
                        class="hover:bg-foreground/5 active:bg-foreground/10 rounded-md"
                    },
                    target: &menu_id,
                    icon(data: mdi::DOTS_VERTICAL)
                )
                dropdown_menu_content(
                    id: &menu_id,
                    align: FloatingAlign::End,
                    for action in permitted {
                        match action.item_type {
                            ActionItemType::Link => dropdown_menu_navigation_item(
                                close: Some(&menu_id),
                                attrs: action.attrs.clone(),
                                (&action.name)
                            ),
                            ActionItemType::Button => dropdown_menu_item(
                                close: Some(&menu_id),
                                attrs: action.attrs.clone(),
                                (&action.name)
                            ),
                        }
                    }
                )
            )
        }
    })
}
