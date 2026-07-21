use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageQueryParams {
    pub offset: Option<usize>,
}

impl ModifyOffset for PageQueryParams {
    fn modify_offset(&mut self, new_offset: usize) {
        self.offset = Some(new_offset);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_page_state() {
        let p = PageState {
            per_page: 50,
            offset: 50,
            total: 61,
        };

        assert_eq!(p.total_pages(), 2);
        assert_eq!(p.current_page(), 2);

        let p = PageState {
            per_page: 50,
            offset: 0,
            total: 61,
        };

        assert_eq!(p.total_pages(), 2);
        assert_eq!(p.current_page(), 1);
        let p = PageState {
            per_page: 50,
            offset: 0,
            total: 50,
        };

        assert_eq!(p.total_pages(), 1);
        assert_eq!(p.current_page(), 1);
    }
}
