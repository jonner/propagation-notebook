use serde::Serialize;
use topcoat::context::{Cx, app_context};

pub const PER_PAGE: usize = 50;

pub trait ModifyOffset: Serialize + std::fmt::Debug {
    fn modify_offset(&mut self, new_offset: usize) -> ();
}

#[derive(Debug, Clone)]
pub struct PageState {
    pub per_page: usize,
    pub offset: usize,
    pub total: usize,
}

impl PageState {
    pub fn new(offset: Option<usize>, per_page: usize, total: usize) -> Self {
        Self {
            per_page,
            offset: offset.unwrap_or_default(),
            total,
        }
    }

    pub fn offset_for_page(&self, page: usize) -> Option<usize> {
        if page > 0 && page <= self.total_pages() {
            Some(self.per_page * page.saturating_sub(1))
        } else {
            None
        }
    }

    pub fn total_pages(&self) -> usize {
        (self.total + self.per_page - 1).div_euclid(self.per_page)
    }

    pub fn current_page(&self) -> usize {
        self.offset.div_euclid(self.per_page) + 1
    }

    pub fn query_with_offset<T: ModifyOffset>(&self, offset: usize, mut params: T) -> String {
        params.modify_offset(offset);
        serde_urlencoded::to_string(&params)
            .map(|qs| format!("?{qs}"))
            .unwrap_or_default()
    }
}

pub fn db(cx: &Cx) -> toasty::Db {
    app_context::<toasty::Db>(cx).clone()
}

pub fn enum_to_string<T: Serialize>(variant: &T) -> String {
    let json_value = serde_json::to_value(variant).expect("Enum variant failed serialization");

    match json_value {
        serde_json::Value::String(s) => s,
        other => other.to_string(),
    }
}
