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
                <div class="slo-check-titles">
                    <span class="slo-check-name">{check.name.clone()}</span>
                    {check
                        .slo_desc
                        .clone()
                        .map(|desc| {
                            view! { <span class="slo-check-subheading">{desc}</span> }
                        })}
                    {check
                        .slo_url
                        .clone()
                        .map(|url| {
                            let link_text = url.clone();
                            view! {
                                <a
                                    class="slo-check-link"
                                    href=url
                                    target="_blank"
                                    rel="noopener noreferrer"
                                >
                                    {link_text}
                                </a>
                            }
                        })}
                </div>

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
