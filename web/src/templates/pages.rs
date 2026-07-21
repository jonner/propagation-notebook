use maud::{Markup, html};

use crate::templates::layout;

pub mod propagation;
pub mod region;
pub mod taxonomy;

pub fn index() -> Markup {
    let content = html! {
        p { "Main page" }
    };
    layout("Propagation Notebook", content)
}
