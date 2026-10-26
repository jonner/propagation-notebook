use libpropagation::auth::{PermissionCode, User};
use topcoat::view::Attributes;

#[derive(Debug, Clone, Copy)]
pub enum ActionItemType {
    Link,
    Button,
}

pub struct Action {
    pub name: String,
    pub permission: Option<PermissionCode>,
    pub attrs: Attributes,
    pub item_type: ActionItemType,
}

pub fn permitted_actions<'a>(user: Option<&User>, actions: &'a [Action]) -> Vec<&'a Action> {
    actions
        .iter()
        .filter(|a| match a.permission {
            None => true,
            Some(permission) => match user {
                Some(user) => user.has_permission(permission),
                None => false,
            },
        })
        .collect()
}
