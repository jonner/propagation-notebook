use libpropagation::{
    citation::Citation,
    propagation::PropagationProcedure,
    region::{ConservationStatus, Origin, RegionalHarvestWindow},
};
use topcoat::view::{attributes, component, view};

use crate::util::{ModifyOffset, PageState};

#[component]
pub async fn citation_list(citations: Vec<&Citation>) -> topcoat::Result {
    view! {
        <ul>
            for citation in citations {
                <li>
                    <a href=(format!("/citation/{}", citation.id))>(&citation.title)</a>
                </li>
            }
        </ul>
    }
}

#[component]
pub async fn pagination_control<'p, T: ModifyOffset + Clone + Sync + Send + 'p>(
    state: &PageState,
    params: &'p T,
) -> topcoat::Result {
    view! {
        <nav class="mt-4">
            <ul class="flex gap-x-2">
                <li>
                    if let Some(offset) = state
                        .offset_for_page(state.current_page() - 1) {
                        <a
                            class="button"
                            href=(state
                                .query_with_offset(offset, params.clone()))
                        >
                            "< Prev"
                        </a>
                    } else {
                        "< Prev"
                    }
                </li>
                <li>
                    if let Some(offset) = state
                        .offset_for_page(state.current_page() + 1) {
                        <a
                            class="button"
                            href=(state
                                .query_with_offset(offset, params.clone()))
                        >
                            "Next >"
                        </a>
                    } else {
                        "Next >"
                    }
                </li>
            </ul>
        </nav>
    }
}

#[component]
pub async fn propagation_details(procedure: &PropagationProcedure) -> topcoat::Result {
    view! {
        <h3>"Type"</h3>
        <div>(procedure.r#type.to_string())</div>
        <h3>"Notes"</h3>
        <div>(procedure.notes.as_deref().unwrap_or_default())</div>
        <h3>"Instructions"</h3>
        <div>(&procedure.instructions)</div>
        <h3>"Citations"</h3>
        <div>
            if !procedure.citations.get().is_empty() {
                citation_list(citations: procedure.citations.get().iter().collect())
            } else {
                "None"
            }
        </div>
    }
}

/// Renders a 52-week harvest window timeline component with week blocks,
/// active harvest window highlight (and optional peak window), month headers,
/// and a vertical line showing the current day/week.
#[component]
pub async fn harvest_timeline(
    window: &RegionalHarvestWindow,
    #[default] current_week: Option<i16>,
) -> topcoat::Result {
    let (Some(start_week), Some(end_week)) = (window.start_week(), window.end_week()) else {
        return view! {};
    };

    let cw =
        current_week.unwrap_or_else(|| jiff::Zoned::now().date().iso_week_date().week().into());

    // Compute left offset percentage for current week marker
    let marker_left_pct = ((cw as f32 - 0.5) / 52.0) * 100.0;

    view! {
        <div class="relative w-full select-none text-xs font-sans">
            <div class="relative w-full">
                // <!-- 52 Week Blocks Grid -->
                <div
                    class="grid grid-cols-[repeat(52,minmax(0,1fr))] gap-[1px] min-h-[1em]"
                >
                    for w in 1..=52 {
                        {
                            let in_window = if start_week <= end_week {
                                w >= start_week && w <= end_week
                            } else {
                                w >= start_week || w <= end_week
                            };

                            let bg_class = if in_window {
                                "bg-leaf/50 border border-leaf/60"
                            } else {
                                "bg-brown/20 border border-brown/22"
                            };

                            <div class=(format!("h-full rounded-[2px] {}", bg_class))></div>
                        }
                    }
                </div>

                // <!-- Current Week Vertical Indicator Marker -->
                <div
                    class="absolute -top-1 -bottom-1 w-[2px] bg-brown z-20"
                    style=(format!("left: {:.2}%;", marker_left_pct))
                ></div>
            </div>
        </div>
    }
}

#[component]
pub async fn origin_badge(origin: Origin) -> topcoat::Result {
    let attrs = attributes! {
        match origin {
            Origin::Introduced => class="introduced",
            Origin::Native => class="native",
            _ => class="",
        }
    };
    view! { <span (attrs)>(origin.to_string())</span> }
}

#[component]
pub async fn conservation_status_badge(status: ConservationStatus) -> topcoat::Result {
    let attrs = attributes! {
        match status {
            ConservationStatus::Endangered => class="endangered",
            ConservationStatus::Threatened => class="threatened",
            ConservationStatus::SpecialConcern => class="specialconcern",
        }
    };
    view! { <span (attrs)>(status.to_string())</span> }
}
