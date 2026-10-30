use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::Serialize;
use toasty::{Db, Deferred};
use uuid::Uuid;
use zxcvbn::Score;

use crate::error::{AuthError, Error};

pub mod dto;

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

#[derive(Debug, thiserror::Error)]
pub enum ChangePasswordError {
    #[error(transparent)]
    Db(#[from] toasty::Error),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("Password doesn't meet requirements: {0}")]
    InvalidPassword(String, Vec<String>),
}

impl User {
    pub fn verify_password(&self, password: &str) -> Result<(), AuthError> {
        verify_password(password, &self.pwhash)
    }

    /// Change the stored password for the given user to `new_password`. This
    /// does not do any verification of the existing password, it is merely
    /// a data update. The caller is tasked with verifying that the user is
    /// allowed to change the password, etc.
    pub async fn change_password(
        &mut self,
        db: &mut Db,
        new_password: &str,
    ) -> Result<(), ChangePasswordError> {
        let entropy = zxcvbn::zxcvbn(new_password, &[&self.username]);
        if entropy.score() < Score::Two {
            let msg = entropy
                .feedback()
                .and_then(|f| f.warning().map(|w| w.to_string()))
                .unwrap_or_else(|| "Password is too weak".to_string());
            let suggestions = entropy
                .feedback()
                .map(|f| f.suggestions().iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            return Err(ChangePasswordError::InvalidPassword(msg, suggestions));
        }
        let new_hash = hash_password(new_password)?;
        self.update().pwhash(new_hash).exec(db).await?;
        Ok(())
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

    pub fn has_permission(&self, permission: PermissionCode) -> bool {
        self.permissions
            .get()
            .iter()
            .find(|p| p.code == permission)
            .is_some()
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

#[derive(
    Debug,
    Clone,
    Copy,
    toasty::Embed,
    PartialEq,
    Eq,
    clap::ValueEnum,
    Serialize,
    strum::EnumIter,
    Hash,
)]
pub enum PermissionCode {
    // ideas: note:create, note:edit, note:vote, propagation:create,
    // propagation:edit, propagation:vote, cleaning:create, cleaning:edit,
    // cleaning:vote,
    #[column(variant = "taxon:sync")]
    TaxonSync,
    #[column(variant = "citation:create")]
    CitationCreate,
    #[column(variant = "citation:edit")]
    CitationEdit,
    #[column(variant = "citation:delete")]
    CitationDelete,
    #[column(variant = "region:sync")]
    RegionSync,
    #[column(variant = "region:create")]
    RegionCreate,
    #[column(variant = "region:delete")]
    RegionDelete,
    #[column(variant = "region:edit")]
    RegionEdit,
    #[column(variant = "taxonpropagation:create")]
    TaxonPropagationCreate,
    #[column(variant = "taxonpropagation:delete")]
    TaxonPropagationDelete,
    #[column(variant = "taxonpropagation:edit")]
    TaxonPropagationEdit,
    #[column(variant = "taxonnote:create")]
    TaxonNoteCreate,
    #[column(variant = "taxonnote:delete")]
    TaxonNoteDelete,
    #[column(variant = "taxonnote:edit")]
    TaxonNoteEdit,
    #[column(variant = "taxoncleaning:create")]
    TaxonCleaningCreate,
    #[column(variant = "taxoncleaning:delete")]
    TaxonCleaningDelete,
    #[column(variant = "taxoncleaning:edit")]
    TaxonCleaningEdit,
    #[column(variant = "admin:console")]
    AdminConsole,
    #[column(variant = "user:list")]
    UserList,
    #[column(variant = "user:create")]
    UserCreate,
    #[column(variant = "user:edit")]
    UserEdit,
    #[column(variant = "user:delete")]
    UserDelete,
    #[column(variant = "user:roles")]
    UserRoles,
    #[column(variant = "role:create")]
    RoleCreate,
    #[column(variant = "role:edit")]
    RoleEdit,
    #[column(variant = "role:delete")]
    RoleDelete,
    #[column(variant = "permission:create")]
    PermissionCreate,
    #[column(variant = "permission:edit")]
    PermissionEdit,
    #[column(variant = "permission:delete")]
    PermissionDelete,
}

impl std::fmt::Display for PermissionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PermissionCode::TaxonSync => "taxon:sync",
                PermissionCode::CitationCreate => "citation:create",
                PermissionCode::CitationEdit => "citation:edit",
                PermissionCode::CitationDelete => "citation:delete",
                PermissionCode::RegionSync => "region:sync",
                PermissionCode::RegionCreate => "region:create",
                PermissionCode::RegionDelete => "region:delete",
                PermissionCode::RegionEdit => "region:edit",
                PermissionCode::TaxonPropagationCreate => "taxonpropagation:create",
                PermissionCode::TaxonPropagationDelete => "taxonpropagation:delete",
                PermissionCode::TaxonPropagationEdit => "taxonpropagation:edit",
                PermissionCode::TaxonNoteCreate => "taxonnote:create",
                PermissionCode::TaxonNoteDelete => "taxonnote:delete",
                PermissionCode::TaxonNoteEdit => "taxonnote:edit",
                PermissionCode::TaxonCleaningCreate => "taxoncleaning:create",
                PermissionCode::TaxonCleaningDelete => "taxoncleaning:delete",
                PermissionCode::TaxonCleaningEdit => "taxoncleaning:edit",
                PermissionCode::AdminConsole => "admin:console",
                PermissionCode::UserList => "user:list",
                PermissionCode::UserCreate => "user:create",
                PermissionCode::UserEdit => "user:edit",
                PermissionCode::UserDelete => "user:delete",
                PermissionCode::UserRoles => "user:roles",
                PermissionCode::RoleCreate => "role:create",
                PermissionCode::RoleEdit => "role:edit",
                PermissionCode::RoleDelete => "role:delete",
                PermissionCode::PermissionCreate => "permission:create",
                PermissionCode::PermissionEdit => "permission:edit",
                PermissionCode::PermissionDelete => "permission:delete",
            }
        )
    }
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
