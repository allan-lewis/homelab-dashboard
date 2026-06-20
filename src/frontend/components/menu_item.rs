use crate::frontend::routing::{push_path, Page};
use leptos::prelude::*;

#[component]
pub fn MenuItem(
    label: &'static str,
    target: Page,
    current_page: ReadSignal<Page>,
    set_current_page: WriteSignal<Page>,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                if current_page.get() == target {
                    "menu-item active"
                } else {
                    "menu-item"
                }
            }
            on:click=move |_| {
                set_current_page.set(target);
                push_path(target.path());
            }
        >
            {label}
        </button>
    }
}

