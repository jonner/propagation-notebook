use serde::Serialize;
use uuid::Uuid;

use crate::auth::Permission;
use crate::auth::PermissionCode;
use crate::auth::Role;
use crate::auth::User;

#[derive(Debug, Serialize)]
pub struct UserCompact {
    pub id: Uuid,
    pub username: String,
}
impl From<&User> for UserCompact {
    fn from(value: &User) -> Self {
        value.clone().into()
    }
}

impl From<User> for UserCompact {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserDetails {
    pub id: Uuid,
    pub username: String,
    pub pwhash: String,
    pub roles: Vec<RoleCompact>,
}

impl From<&User> for UserDetails {
    fn from(value: &User) -> Self {
        value.clone().into()
    }
}

impl From<User> for UserDetails {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            pwhash: value.pwhash,
            roles: value.roles.get().iter().map(|r| r.into()).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RoleCompact {
    pub id: Uuid,
    pub name: String,
}
impl From<&Role> for RoleCompact {
    fn from(value: &Role) -> Self {
        value.clone().into()
    }
}

impl From<Role> for RoleCompact {
    fn from(value: Role) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RoleDetails {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<PermissionCompact>,
}

impl From<&Role> for RoleDetails {
    fn from(value: &Role) -> Self {
        value.clone().into()
    }
}

impl From<Role> for RoleDetails {
    fn from(value: Role) -> Self {
        tracing::debug!(?value);
        Self {
            id: value.id,
            name: value.name,
            permissions: value.permissions.get().iter().map(|p| p.into()).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PermissionCompact {
    pub id: Uuid,
    pub code: PermissionCode,
}
impl From<&Permission> for PermissionCompact {
    fn from(value: &Permission) -> Self {
        value.clone().into()
    }
}

impl From<Permission> for PermissionCompact {
    fn from(value: Permission) -> Self {
        Self {
            id: value.id,
            code: value.code,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PermissionDetails {
    pub id: Uuid,
    pub code: PermissionCode,
    pub description: String,
}

impl From<&Permission> for PermissionDetails {
    fn from(value: &Permission) -> Self {
        value.clone().into()
    }
}

impl From<Permission> for PermissionDetails {
    fn from(value: Permission) -> Self {
        Self {
            id: value.id,
            code: value.code,
            description: value.description,
        }
    }
}
