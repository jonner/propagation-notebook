use libpropagation::auth::dto::{
    PermissionCompact, PermissionDetails, RoleCompact, RoleDetails, UserCompact, UserDetails,
};

use crate::{style, util::enum_to_string};

pub struct UserDetailsView<'a> {
    user: &'a UserDetails,
}

impl<'a> UserDetailsView<'a> {
    pub fn new(user: &'a UserDetails) -> Self {
        Self { user }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.user.id.to_string()]);
        tbuilder.push_record(["Username", &self.user.username.to_string()]);
        tbuilder.push_record(["Password Hash", &self.user.pwhash]);
        tbuilder.push_record(["Roles", &{
            if !self.user.roles.is_empty() {
                let mut inner = tabled::builder::Builder::default();
                inner.push_record(["ID", "Name"]);
                for role in self.user.roles.iter() {
                    inner.push_record([&role.id.to_string(), &role.name])
                }
                inner.build().with(style::ListTable).to_string()
            } else {
                "-".to_string()
            }
        }]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}

pub struct UserListView<'a> {
    users: &'a Vec<UserCompact>,
}

impl<'a> UserListView<'a> {
    pub fn new(users: &'a Vec<UserCompact>) -> Self {
        Self { users }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", "Username"]);
        for user in self.users {
            tbuilder.push_record([user.id.to_string(), user.username.to_string()]);
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}

pub struct RoleDetailsView<'a> {
    role: &'a RoleDetails,
}

impl<'a> RoleDetailsView<'a> {
    pub fn new(role: &'a RoleDetails) -> Self {
        Self { role }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.role.id.to_string()]);
        tbuilder.push_record(["Name", &self.role.name.to_string()]);
        tbuilder.push_record(["Permissions", &{
            if !self.role.permissions.is_empty() {
                let mut inner = tabled::builder::Builder::default();
                inner.push_record(["ID", "Name"]);
                for perm in self.role.permissions.iter() {
                    inner.push_record([perm.id.to_string(), enum_to_string(&perm.code)])
                }
                inner.build().with(style::ListTable).to_string()
            } else {
                "-".to_string()
            }
        }]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}

pub struct RoleListView<'a> {
    roles: &'a Vec<RoleCompact>,
}

impl<'a> RoleListView<'a> {
    pub fn new(roles: &'a Vec<RoleCompact>) -> Self {
        Self { roles }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", "code"]);
        for user in self.roles {
            tbuilder.push_record([user.id.to_string(), user.name.to_string()]);
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}

pub struct PermissionDetailsView<'a> {
    permission: &'a PermissionDetails,
}

impl<'a> PermissionDetailsView<'a> {
    pub fn new(permission: &'a PermissionDetails) -> Self {
        Self { permission }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.permission.id.to_string()]);
        tbuilder.push_record(["Code", &enum_to_string(&self.permission.code)]);
        tbuilder.push_record(["Description", &self.permission.description]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}

pub struct PermissionListView<'a> {
    permissions: &'a Vec<PermissionCompact>,
}

impl<'a> PermissionListView<'a> {
    pub fn new(permissions: &'a Vec<PermissionCompact>) -> Self {
        Self { permissions }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", "code"]);
        for user in self.permissions {
            tbuilder.push_record([user.id.to_string(), enum_to_string(&user.code)]);
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}
