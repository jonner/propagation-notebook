use serde::Deserialize;

pub const PER_PAGE: usize = 50;

pub struct PageState {
    pub per_page: usize,
    pub offset: usize,
    pub total: u64,
}

impl PageState {
    pub fn has_next(&self) -> bool {
        self.total > (self.offset + self.per_page).try_into().unwrap()
    }

    pub fn next_offset(&self) -> usize {
        (self.offset + self.per_page).min(self.total.try_into().unwrap())
    }

    pub fn prev_offset(&self) -> usize {
        self.offset.saturating_sub(self.per_page)
    }

    pub fn has_prev(&self) -> bool {
        self.offset > 0
    }

    pub fn n_pages(&self) -> u64 {
        self.total.rem_euclid(self.per_page.try_into().unwrap())
    }

    pub fn page_num(&self) -> u64 {
        (self.offset.div_euclid(self.per_page) + 1)
            .try_into()
            .unwrap()
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct PageQueryParams {
    pub offset: Option<usize>,
}
