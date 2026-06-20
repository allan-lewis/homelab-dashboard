use crate::frontend::components::menu_item::MenuItem;
use crate::frontend::components::theme_selector::ThemeSelector;
use crate::frontend::routing::Page;
use crate::frontend::theme::ThemeMode;
use leptos::prelude::*;

#[component]
pub fn SideMenu(
    menu_open: ReadSignal<bool>,
    current_page: ReadSignal<Page>,
    set_current_page: WriteSignal<Page>,
    theme_mode: ReadSignal<ThemeMode>,
    set_theme_mode: WriteSignal<ThemeMode>,
) -> impl IntoView {
    view! {
        <aside class=move || {
            if menu_open.get() {
                "side-menu open"
            } else {
                "side-menu closed"
            }
        }>
            <nav>
                <MenuItem label="Overview" target=Page::Overview current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Hosts" target=Page::Hosts current_page=current_page set_current_page=set_current_page />
                <MenuItem label="NixOS Generations" target=Page::Generations current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Uptime" target=Page::Uptime current_page=current_page set_current_page=set_current_page />
            </nav>

            <ThemeSelector
                theme_mode=theme_mode
                set_theme_mode=set_theme_mode
            />
        </aside>
    }
}

