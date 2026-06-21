use leptos::prelude::*;

use crate::frontend::components::app_header::AppHeader;
use crate::frontend::components::side_menu::SideMenu;
use crate::frontend::current_page::CurrentPage;
use crate::frontend::models::User;
use crate::frontend::routing::{current_path, Page};
use crate::frontend::theme::theme_mode_from_storage;

#[component]
pub fn AppShell(
    current_user: User,
    menu_open: ReadSignal<bool>,
    set_menu_open: WriteSignal<bool>,
) -> impl IntoView {
    let name = current_user.name.clone();
    let email = current_user.email.clone();
    let (current_page, set_current_page) = signal(Page::from_path(&current_path()));
    let initial_theme_mode = theme_mode_from_storage();

    let (theme_mode, set_theme_mode) = signal(initial_theme_mode);

    view! {
        <main class="app-shell">
            <AppHeader
                name=name.clone()
                email=email
                menu_open=menu_open
                set_menu_open=set_menu_open
            />

            <div class="app-body">
                <SideMenu
                    menu_open=menu_open
                    current_page=current_page
                    set_current_page=set_current_page
                    theme_mode=theme_mode
                    set_theme_mode=set_theme_mode
                />

                <CurrentPage
                    current_page=current_page
                />
            </div>
        </main>
    }
}

