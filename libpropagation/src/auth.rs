use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::Serialize;
use toasty::{Db, Deferred};
use uuid::Uuid;

use crate::error::{AuthError, Error};

#[derive(Debug, Clone, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    #[unique]
    pub username: String,
    #[index]
    pub pwhash: String,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
    pub last_login_at: Option<jiff::Timestamp>,

    #[has_many]
    pub user_roles: Deferred<Vec<UserRole>>,
    #[has_many(via=user_roles.role)]
    pub roles: Deferred<Vec<Role>>,
    #[has_many(via=user_roles.role.role_permissions.permission)]
    pub permissions: Deferred<Vec<Permission>>,
    #[has_many]
    pub sessions: toasty::Deferred<Vec<Session>>,
    #[has_one]
    pub profile: toasty::Deferred<Option<UserProfile>>,
    #[has_many]
    pub emails: toasty::Deferred<Vec<UserEmail>>,
}

impl User {
    pub fn verify_password(&self, password: &str) -> Result<(), AuthError> {
        verify_password(password, &self.pwhash)
    }

    /// This doesn't validate the password or provide any security. It just
    /// marks the user as logged in. It is expected that the caller verifies the
    /// password before calling this function
    pub async fn login(
        &mut self,
        db: &mut Db,
        session_hash: &[u8],
        expiration: jiff::Timestamp,
    ) -> Result<(), Error> {
        Session::create()
            .token_hash(Vec::from(session_hash))
            .expires_at(expiration)
            .user_id(self.id)
            .exec(db)
            .await?;
        self.update()
            .last_login_at(jiff::Timestamp::now())
            .exec(db)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, toasty::Model)]
pub struct UserProfile {
    #[key]
    pub user_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,

    #[belongs_to]
    pub user: Deferred<User>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct UserEmail {
    #[key]
    pub id: Uuid,
    #[index]
    pub user_id: Uuid,
    #[index]
    pub address: String,
    #[default(false)]
    pub confirmed: bool,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,

    #[belongs_to]
    pub user: Deferred<User>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Session {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    #[unique]
    pub token_hash: Vec<u8>,
    #[index]
    pub user_id: Uuid,
    pub expires_at: jiff::Timestamp,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,

    #[belongs_to]
    pub user: toasty::Deferred<User>,
}

// join table for users and roles
#[derive(Debug, Clone, toasty::Model)]
pub struct UserRole {
    #[key]
    #[index]
    pub user_id: Uuid,
    #[key]
    #[index]
    pub role_id: Uuid,
    #[auto]
    pub created_at: jiff::Timestamp,

    #[belongs_to(key=user_id, references=id)]
    pub user: Deferred<User>,
    #[belongs_to(key=role_id, references=id)]
    pub role: Deferred<Role>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Role {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub name: String,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,

    #[has_many]
    pub role_permissions: Deferred<Vec<RolePermission>>,
    #[has_many(via=role_permissions.permission)]
    pub permissions: Deferred<Vec<Permission>>,
    #[has_many]
    pub user_roles: Deferred<Vec<UserRole>>,
    #[has_many(via=user_roles.user)]
    pub users: Deferred<Vec<User>>,
}

// join table for roles and permissions
#[derive(Debug, Clone, toasty::Model)]
pub struct RolePermission {
    #[key]
    #[index]
    pub role_id: Uuid,
    #[key]
    #[index]
    pub permission_id: Uuid,
    #[auto]
    pub created_at: jiff::Timestamp,

    #[belongs_to(key=permission_id, references=id)]
    pub permission: Deferred<Permission>,
    #[belongs_to(key=role_id, references=id)]
    pub role: Deferred<Role>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Permission {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    #[unique]
    pub code: PermissionCode,
    pub description: String,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,

    #[has_many]
    pub role_permissions: Deferred<Vec<RolePermission>>,
    #[has_many(via=role_permissions.role)]
    pub roles: Deferred<Vec<Role>>,
}

#[derive(Debug, Clone, toasty::Embed, PartialEq, clap::ValueEnum, Serialize)]
pub enum PermissionCode {
    // ideas: note:create, note:edit, note:vote, propagation:create,
    // propagation:edit, propagation:vote, cleaning:create, cleaning:edit,
    // cleaning:vote,
    #[column(variant = "taxon:sync")]
    TaxonSync,
}

pub fn hash_password(pw: &str) -> Result<String, AuthError> {
    let hasher = Argon2::default();
    Ok(hasher.hash_password(pw.as_bytes())?.to_string())
}

pub fn verify_password(pw: &str, expected_pwhash: &str) -> Result<(), AuthError> {
    let hasher = Argon2::default();
    let expected_hash =
        PasswordHash::new(expected_pwhash).map_err(|_| AuthError::InvalidPasswordHash)?;
    hasher
        .verify_password(pw.as_bytes(), &expected_hash)
        .map_err(|e| e.into())
}

pub mod dto {
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
}
