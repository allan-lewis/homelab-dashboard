use leptos::prelude::*;

use crate::frontend::slo::slo_day_box_class;

#[component]
pub fn SloDayBox(date: String, pct: Option<f64>) -> impl IntoView {
    let title = match pct {
        Some(pct) => format!("{date}: {:.2}%", pct * 100.0),
        None => format!("{date}: no data"),
    };

    view! { <div class=slo_day_box_class(pct) title=title></div> }
}
