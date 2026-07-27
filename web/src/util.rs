use libpropagation::{
    collecting::CleaningProcedure,
    propagation::PropagationProcedure,
    region::{Region, RegionalTaxonStatus},
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use serde::Serialize;
use topcoat::{
    context::{Cx, app_context},
    router::{path_param, query_params},
    view::{component, view},
};

#[path_param(error = bad_request)]
pub(crate) struct TaxonId(u64);

#[path_param(error = bad_request)]
pub(crate) struct CitationId(u64);

#[path_param(error = bad_request)]
pub(crate) struct RegionId(u64);

#[path_param(error= bad_request)]
pub struct PropagationId(u64);

#[path_param(error = bad_request)]
pub struct CleaningId(u64);

pub trait Path {
    fn path(&self) -> String;
}

impl Path for CleaningProcedure {
    fn path(&self) -> String {
        format!("/taxa/{}/cleaning/{}", self.taxon_id, self.id)
    }
}

impl Path for PropagationProcedure {
    fn path(&self) -> String {
        format!("/propagation/{}", self.id)
    }
}

impl Path for TaxonPropagationProcedure {
    fn path(&self) -> String {
        format!(
            "/taxa/{}/propagation/{}",
            self.taxon_id, self.propagation_id
        )
    }
}

impl Path for Taxon {
    fn path(&self) -> String {
        format!("/taxa/{}", self.id)
    }
}

impl Path for Region {
    fn path(&self) -> String {
        format!("/regions/{}", self.id)
    }
}

impl Path for RegionalTaxonStatus {
    fn path(&self) -> String {
        format!("/regions/{}/taxa/{}", self.region_id, self.taxon_id)
    }
}

#[derive(Debug)]
pub enum Dimension {
    Pixels(u64),
    Percent(u64),
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dimension::Pixels(v) => write!(f, "{v}px"),
            Dimension::Percent(v) => write!(f, "{v}%"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[query_params(error = bad_request)]
pub struct PageQueryParams {
    pub offset: Option<usize>,
}

impl ModifyOffset for PageQueryParams {
    fn modify_offset(&mut self, new_offset: usize) {
        self.offset = Some(new_offset);
    }
}

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
    pub fn new(offset: Option<usize>, total: usize) -> Self {
        Self {
            per_page: PER_PAGE,
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

#[component]
pub async fn pagination_control<'p, T: ModifyOffset + Clone + Sync + Send + 'p>(
    state: &PageState,
    params: &'p T,
) -> topcoat::Result {
    view! {
        <nav>
            <ul>
                <li>
                    if let Some(offset) = state
                        .offset_for_page(state.current_page() - 1) {
                        <a
                            href=(state
                                .query_with_offset(offset, params.clone()))
                        >
                            "< Prev"
                        </a>
                    }
                </li>
                <li>
                    if let Some(offset) = state
                        .offset_for_page(state.current_page() + 1) {
                        <a
                            href=(state
                                .query_with_offset(offset, params.clone()))
                        >
                            "Next >"
                        </a>
                    }
                </li>
            </ul>
        </nav>
    }
}

pub fn db(cx: &Cx) -> toasty::Db {
    app_context::<toasty::Db>(cx).clone()
}
