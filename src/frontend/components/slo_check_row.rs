use leptos::prelude::*;

use crate::frontend::components::slo_day_box::SloDayBox;
use crate::frontend::models::SloCheckStatus;
use crate::frontend::slo::slo_pill_class;

#[component]
pub fn SloCheckRow(check: SloCheckStatus) -> impl IntoView {
    let pct_28d = check.pct_28d;

    view! {
        <div class="slo-check-row">
            <div class="slo-check-header">
                <span class="slo-check-name">{check.name.clone()}</span>

                <span class=slo_pill_class(pct_28d)>{format!("{:.3}%", pct_28d * 100.0)}</span>
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
