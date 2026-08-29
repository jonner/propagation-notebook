use libpropagation::taxonomy::Taxon;
use topcoat::{
    Result,
    context::Cx,
    icon::{icon, iconify::iconify_icon},
    router::{href, query_params},
    view::{Attributes, View, attributes, class, component, view},
};

use crate::taxa::TaxaListParams;

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

pub struct Breadcrumb {
    pub url: Option<String>,
    pub text: String,
}

#[component]
pub async fn breadcrumbs(
    items: Vec<Breadcrumb>,
    #[default] ellipsize: Option<usize>,
) -> topcoat::Result {
    let do_ellipsize = ellipsize.map(|n| items.len() > n + 1) == Some(true);
    let mut breadcrumb_iter = items.into_iter();
    let Some(root) = breadcrumb_iter.next() else {
        return view! {};
    };
    let rest: Vec<_> = if let Some(ellipsize) = ellipsize {
        breadcrumb_iter.rev().take(ellipsize).rev().collect()
    } else {
        breadcrumb_iter.collect()
    };
    view! {
        breadcrumb(
            breadcrumb_list(
                breadcrumb_item(breadcrumb_link(attrs: attributes! { href=(root.url) }, (&root.text)))
                if do_ellipsize {
                    breadcrumb_separator()
                    breadcrumb_ellipsis()
                }
                for item in rest {
                    breadcrumb_separator()
                    if let Some(url) = item.url {
                        breadcrumb_item(breadcrumb_link(attrs: attributes! { href=(url) }, (item.text)))
                    } else {
                        breadcrumb_item(breadcrumb_page((item.text)))
                    }
                }
            )
        )
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
    let f = |t: &Taxon, link: bool| -> Breadcrumb {
        Breadcrumb {
            url: link.then_some(
                href!(crate::taxa::taxonomy)
                    .query(TaxaListParams {
                        offset: None,
                        parent: Some(t.id),
                        fmt: params.fmt,
                        region: params.region,
                    })
                    .resolve(cx),
            ),
            text: t.complete_name.clone(),
        }
    };
    let mut ancestors: Vec<_> = items
        .iter()
        .take(items.len() - 1)
        .map(|taxon| f(taxon, true))
        .collect();
    ancestors.push(f(items.last().unwrap(), link_final));
    view! { breadcrumbs(items: ancestors, ellipsize: ellipsize) }
}
