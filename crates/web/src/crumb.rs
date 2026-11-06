use topcoat::{
    context::{AsyncCxFn, BoxCxFn, Cx, iter_context},
    router::request::uri,
    view::{View, attributes, component, view},
};

use crate::components::breadcrumb::*;

pub struct Crumb(BoxCxFn<topcoat::Result<(String, Option<String>)>>);

impl Crumb {
    pub fn new(details: impl AsyncCxFn<topcoat::Result<(String, Option<String>)>>) -> Self {
        Self(BoxCxFn::new(details))
    }
}

#[component]
pub async fn crumbs(cx: &Cx) -> topcoat::Result<impl View> {
    let crumbs_iter = iter_context::<Crumb>(cx).collect::<Vec<_>>();
    Ok(view! {
        breadcrumb(
            breadcrumb_list(
                for (n, crumb) in crumbs_iter.into_iter().rev().enumerate() {
                    if n != 0 {
                        breadcrumb_separator()
                    }
                    breadcrumb_item(
                        match crumb.0.call(cx).await? {
                            (label, Some(link)) if link != uri(cx).to_string() => breadcrumb_link(
                                attrs: attributes! { href=(link) },
                                (label)
                            ),
                            (label, _) => breadcrumb_page((label)),
                        }
                    )
                }
            )
        )
    })
}
