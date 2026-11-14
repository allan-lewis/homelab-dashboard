use leptos::prelude::*;

use crate::frontend::components::slo_day_box::SloDayBox;
use crate::frontend::models::{LiveState, SloCheckStatus};
use crate::frontend::slo::{live_pill_class, live_pill_label, slo_pill_class};

#[component]
pub fn SloCheckRow(check: SloCheckStatus, live_state: Option<LiveState>) -> impl IntoView {
    let pct_28d = check.pct_28d;

    view! {
        <div class="slo-check-row">
            <div class="slo-check-header">
                <span class="slo-check-name">{check.name.clone()}</span>

                <div class="slo-check-pills">
                    {live_state
                        .map(|state| {
                            view! {
                                <span class=live_pill_class(&state)>{live_pill_label(&state)}</span>
                            }
                        })}

                    <span class=slo_pill_class(pct_28d)>{format!("{:.3}%", pct_28d * 100.0)}</span>
                </div>
            </div>

            <div class="slo-day-grid">
                {check
                    .daily
                    .into_iter()
                    .map(|day| {
                        view! { <SloDayBox date=day.date pct=day.pct /> }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}
