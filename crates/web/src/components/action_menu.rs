use topcoat::{
    context::Cx,
    icon::icon,
    view::{Attributes, View, component, element_id, view},
};

use crate::{
    actions::{Action, ActionItemType},
    assets::mdi,
    components::{dropdown_menu::*, floating::FloatingAlign},
};

#[component]
pub async fn action_menu(
    cx: &Cx,
    #[default] attrs: Attributes,
    actions: &[&Action],
) -> topcoat::Result<impl View> {
    let menu_id = element_id(cx);
    Ok(view! {
        dropdown_menu(
            attrs: attrs,
            dropdown_menu_trigger(target: &menu_id, icon(data: mdi::DOTS_VERTICAL))
            dropdown_menu_content(
                id: &menu_id,
                align: FloatingAlign::End,
                for action in actions {
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
    })
}
