use topcoat::view::{Attributes, class, component, view};

use libpropagation::region::{ConservationStatus, Origin};

use crate::components::tooltip::{tooltip, tooltip_content};

#[component]
pub async fn origin_badge(origin: Origin, #[default] mut attrs: Attributes) -> topcoat::Result {
    let vals = match origin {
        Origin::Introduced => Some(("introduced", "IN", "Introduced")),
        Origin::Unknown => Some(("unknown", "UN", "Unknown origin")),
        Origin::Native => None,
    };
    if let Some((klass, text, tooltip_text)) = vals {
        view! {
            tooltip(
            <div
                class=(class!(klass, "badge", attrs.remove("class")))
                (attrs)
            >
                (text)
            </div>
                tooltip_content((tooltip_text))
        )
        }
    } else {
        view! {}
    }
}

#[component]
pub async fn conservation_status_badge(
    status: ConservationStatus,
    #[default] mut attrs: Attributes,
) -> topcoat::Result {
    let (klass, text, tooltip_text) = match status {
        ConservationStatus::Endangered => ("endangered", "EN", "Endangered"),
        ConservationStatus::Threatened => ("threatened", "TH", "Threatened"),
        ConservationStatus::SpecialConcern => ("specialconcern", "SC", "Special Concern"),
    };
    view! {
        tooltip(
        <div
            class=(class!(klass, "badge", attrs.remove("class")))
            (attrs)
        >
            (text)
        </div>
            tooltip_content((tooltip_text))
    )
    }
}
