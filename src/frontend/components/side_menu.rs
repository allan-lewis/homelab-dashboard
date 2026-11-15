use crate::frontend::components::menu_icons::MenuIcon;
use crate::frontend::components::menu_item::MenuItem;
use crate::frontend::menu_state::save_menu_open;
use crate::frontend::routing::Page;
use leptos::prelude::*;

#[component]
pub fn SideMenu(
    current_page: ReadSignal<Page>,
    set_current_page: WriteSignal<Page>,
    menu_open: ReadSignal<bool>,
    set_menu_open: WriteSignal<bool>,
) -> impl IntoView {
    let toggle_menu = move |_| {
        let next = !menu_open.get();
        set_menu_open.set(next);
        save_menu_open(next);
    };

    view! {
        <aside class=move || {
            if menu_open.get() { "side-menu open" } else { "side-menu closed" }
        }>
            <nav>
                <MenuItem label="Overview" icon=MenuIcon::Overview target=Page::Overview current_page=current_page set_current_page=set_current_page />
                <MenuItem label="SLOs" icon=MenuIcon::Slo target=Page::Slo current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Alerts" icon=MenuIcon::Alerts target=Page::Alerts current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Hosts" icon=MenuIcon::Hosts target=Page::Hosts current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Prometheus Targets" icon=MenuIcon::Targets target=Page::Up current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Certificates" icon=MenuIcon::Certificates target=Page::Certificates current_page=current_page set_current_page=set_current_page />
                <MenuItem label="NixOS Generations" icon=MenuIcon::Generations target=Page::Generations current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Scheduled Tasks" icon=MenuIcon::Tasks target=Page::Tasks current_page=current_page set_current_page=set_current_page />
            </nav>

            <button
                class="menu-toggle"
                on:click=toggle_menu
                title=move || if menu_open.get() { "Collapse menu" } else { "Expand menu" }
            >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="3" y="4" width="18" height="16" rx="2" />
                    <path d="M9 4v16" />
                    <rect class="menu-toggle-rail" x="4" y="5" width="4" height="14" rx="1" stroke="none" />
                </svg>
            </button>
        </aside>
    }
}
