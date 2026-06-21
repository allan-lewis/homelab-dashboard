use leptos::prelude::*;

#[component]
pub fn SummaryLine(
    label: &'static str,
    count: usize,
    pill_class: &'static str,
) -> impl IntoView {
    view! {
        <div class="summary-line">
            <span>{label}</span>

            <span class=pill_class>
                {count}
            </span>
        </div>
    }
}
