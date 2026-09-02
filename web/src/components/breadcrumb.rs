use libpropagation::taxonomy::Taxon;
use topcoat::{
    Result,
    context::Cx,
    icon::{icon, iconify::iconify_icon},
    router::{href, query_params},
    view::{Attributes, View, attributes, class, component, view},
};

use crate::{components::badge::*, taxa::TaxaListParams};

/// A breadcrumb component: the trail from the site's root to the current
/// page.
///
/// The trail is a `<nav>` labelled as a breadcrumb, holding a
/// [`breadcrumb_list`] of [`breadcrumb_item`]s. Each item is a
/// [`breadcrumb_link`], except the last, which is the current page and is a
/// [`breadcrumb_page`] instead. The `attrs` (such as `class`) are forwarded to
/// the `<nav>`; a `class` among them is appended to the computed classes.
///
/// ```ignore
/// view! {
///     breadcrumb(
///         breadcrumb_list(
///             breadcrumb_item(
///                 breadcrumb_link(attrs: attributes! { href="/" }, "Home")
///             )
///             breadcrumb_separator()
///             breadcrumb_item(breadcrumb_page("Settings"))
///         )
///     )
/// }
/// ```
#[component]
pub async fn breadcrumb(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <nav aria-label="breadcrumb" class=(attrs.remove("class")) (attrs)>(child)</nav>
    }
}

/// The ordered list of steps in a [`breadcrumb`].
///
/// The steps wrap onto another line rather than overflowing when the trail
/// outgrows its container.
#[component]
pub async fn breadcrumb_list(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <ol
            class=(class!(
                "flex flex-wrap items-center gap-2 text-sm text-muted-foreground",
                attrs.remove("class"),
            ))
            (attrs)
        >
            (child)
        </ol>
    }
}

/// One step of a [`breadcrumb_list`], holding a [`breadcrumb_link`] or a
/// [`breadcrumb_page`].
#[component]
pub async fn breadcrumb_item(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <li
            class=(class!("inline-flex items-center gap-2", attrs.remove("class")))
            (attrs)
        >
            (child)
        </li>
    }
}

/// A step of the trail that leads somewhere: an `<a>` to an ancestor of the
/// current page.
///
/// It takes the list's muted color at rest and the full foreground color on
/// hover. Pass the `href` among the `attrs`.
#[component]
pub async fn breadcrumb_link(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <a
            class=(class!(
                "transition-colors hover:text-foreground",
                attrs.remove("class"),
            ))
            (attrs)
        >
            (child)
        </a>
    }
}

/// The last step of the trail: the current page, which is not a link.
///
/// It carries `aria-current="page"`, so assistive technology announces it as
/// where the reader is.
#[component]
pub async fn breadcrumb_page(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <span
            aria-current="page"
            class=(class!("font-medium text-foreground", attrs.remove("class")))
            (attrs)
        >
            (child)
        </span>
    }
}

/// The divider between two steps of a [`breadcrumb_list`].
///
/// It is a chevron pointing along the trail, hidden from assistive
/// technology, which reads the steps as a list without it.
#[component]
pub async fn breadcrumb_separator(#[default] mut attrs: Attributes) -> Result {
    view! {
        <li aria-hidden="true" class=(attrs.remove("class")) (attrs)>
            icon(
                data: iconify_icon!("mdi:chevron-right"),
                attrs: attributes! { class="size-3.5" }
            )
        </li>
    }
}

/// A stand-in for the steps left out of a long trail.
///
/// It shows an ellipsis in place of the collapsed steps. The glyph itself
/// says nothing to assistive technology, so a word standing in for it is read
/// out instead.
#[component]
pub async fn breadcrumb_ellipsis(#[default] mut attrs: Attributes) -> Result {
    view! {
        <span class=(class!("flex items-center", attrs.remove("class"))) (attrs)>
            icon(
                data: iconify_icon!("mdi:more-horiz"),
                attrs: attributes! { class="size-4" }
            )
            <span class="sr-only">"More"</span>
        </span>
    }
}

#[component]
pub async fn ancestor_breadcrumbs(
    cx: &Cx,
    items: &[&Taxon],
    #[default] link_final: bool,
    #[default(Some(2))] ellipsize: Option<usize>,
) -> topcoat::Result {
    let params = query_params::<TaxaListParams>(cx)
        .cloned()
        .unwrap_or_default();
    let mut item_iter = items.iter();
    let last = item_iter.next_back();
    let root = item_iter.next();
    let middle_items: Vec<_> = item_iter.collect();

    let total_len = items.len();
    let do_ellipsize = ellipsize.map(|limit| total_len > limit).unwrap_or(false);

    let middle_to_render = if do_ellipsize {
        if let Some(limit) = ellipsize {
            middle_items
                .iter()
                .rev()
                .take(limit.saturating_sub(1))
                .rev()
                .copied()
                .collect()
        } else {
            middle_items
        }
    } else {
        middle_items
    };

    view! {
        breadcrumb(
            breadcrumb_list(
                if let Some(taxon) = root {
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! {
                                href=(href!(crate::taxa::taxonomy)
                                    .query(TaxaListParams {
                                        offset: None,
                                        parent: Some(taxon.id),
                                        fmt: params.fmt,
                                        region: params.region,
                                    }))
                            },
                            (&taxon.complete_name)
                        )
                    )
                }
                if do_ellipsize {
                    breadcrumb_separator()
                    breadcrumb_ellipsis()
                }
                for taxon in middle_to_render {
                    breadcrumb_separator()
                    breadcrumb_item(
                        breadcrumb_link(
                            attrs: attributes! {
                                href=(href!(crate::taxa::taxonomy)
                                    .query(TaxaListParams {
                                        offset: None,
                                        parent: Some(taxon.id),
                                        fmt: params.fmt,
                                        region: params.region,
                                    }))
                            },
                            (&taxon.complete_name)
                        )
                    )
                }

                if let Some(taxon) = last {
                    if root.is_some() {
                        breadcrumb_separator()
                    }
                    breadcrumb_item(
                        if link_final {
                            breadcrumb_link(
                                attrs: attributes! {
                                    href=(href!(crate::taxa::taxonomy)
                                        .query(TaxaListParams {
                                            offset: None,
                                            parent: Some(taxon.id),
                                            fmt: params.fmt,
                                            region: params.region,
                                        }))
                                },
                                (&taxon.complete_name)
                            )
                        } else {
                            breadcrumb_page(
                                (&taxon.complete_name)
                                badge(
                                    variant: BadgeVariant::Secondary,
                                    attrs: attributes! { class="mx-3" },
                                    (taxon.rank.to_string())
                                )
                            )
                        }
                    )
                }
            )
        )
    }
}
