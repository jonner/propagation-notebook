use std::convert::From;

use serde::Serialize;
use toasty::Deferred;

#[derive(Serialize, Debug, Clone)]
pub struct ObjectReference {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ObjectReference {
    pub fn from_deferred<T>(deferred: Deferred<T>, default_id: u64) -> Self
    where
        Self: for<'a> From<&'a T>,
    {
        match deferred.is_unloaded() {
            true => Self {
                id: default_id,
                name: None,
            },
            false => deferred.get().into(),
        }
    }

    pub fn from_deferred_option<T>(
        deferred: Deferred<Option<T>>,
        default_id: Option<u64>,
    ) -> Option<Self>
    where
        Self: for<'a> From<&'a T>,
    {
        match deferred.is_unloaded() {
            true => default_id.map(|id| Self { id, name: None }),
            false => deferred.get().as_ref().map(|val| val.into()),
        }
    }
}

impl std::fmt::Display for ObjectReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}: {}", self.id, name)
        } else {
            write!(f, "{}", self.id)
        }
    }
}
