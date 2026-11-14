use crate::frontend::components::menu_icons::MenuIcon;
use crate::frontend::routing::{push_path, Page};
use leptos::prelude::*;

#[component]
pub fn MenuItem(
    label: &'static str,
    icon: MenuIcon,
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
            title=label
            on:click=move |_| {
                set_current_page.set(target);
                push_path(target.path());
            }
        >
            <span class="menu-item-label">{label}</span>
            <span class="menu-item-icon">{icon.view()}</span>
        </button>
    }
}
