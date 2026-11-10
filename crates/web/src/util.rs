use topcoat::{
    context::Cx,
    icon::icon,
    router::request::uri,
    runtime::{Event, Signal},
    view::{Attributes, View, attributes, component, view},
};

use crate::{
    assets::mdi,
    components::{button::*, input::input, textarea::textarea},
};

pub const PER_PAGE: usize = 50;

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
}

pub struct Query(Vec<(String, String)>);

impl Query {
    pub fn from_cx(cx: &Cx) -> Self {
        Self(serde_urlencoded::from_str(uri(cx).query().unwrap_or("")).unwrap_or_default())
    }
    pub fn with(&self, key: &str, value: impl ToString) -> String {
        let mut pairs = self.0.clone();
        pairs.retain(|(k, _)| k != key);
        pairs.push((key.into(), value.to_string()));
        serde_urlencoded::to_string(&pairs)
            .map(|qs| format!("?{qs}"))
            .unwrap_or_default()
    }
}

/// must provide a @handler in 'attrs' to actually set the value
#[component]
pub(crate) async fn editable_text(
    cx: &Cx,
    editable: Signal<bool>,
    val: Option<&String>,
    #[default] multiline: bool,
    #[default] attrs: Attributes,
) -> topcoat::Result<impl View> {
    let default_attrs = attributes! {
        @change=$(|_e: Event| editable.set(false))
        @keydown=$(|e: Event| {
            if e.key == "Escape" {
                editable.set(false)
            }
        })
    };
    Ok(view! {
        <div class="flex items-center gap-2">
            if editable.get() {
                if multiline {
                    textarea(attrs: attributes! { (default_attrs) (attrs) }, (val))
                } else {
                    input(
                        attrs: attributes! { (default_attrs) (attrs) type="text" value=(val) }
                    )
                }
            } else {
                if let Some(s) = val {
                    (s)
                } else {
                    <span class="italic">"None"</span>
                }
            }
            button(
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Sm,
                attrs: attributes! {
                    type="button"
                    @click=$(|e: Event| {
                        e.prevent_default();
                        editable.toggle()
                    })
                },
                icon(data: mdi::EDIT, label: "Edit")
            )
        </div>
    })
}
