use leptos::prelude::*;

use crate::frontend::components::app_header::AppHeader;
use crate::frontend::components::side_menu::SideMenu;
use crate::frontend::components::site_footer::SiteFooter;
use crate::frontend::current_page::CurrentPage;
use crate::frontend::models::User;
use crate::frontend::routing::{current_path, Page};
use crate::frontend::theme::theme_mode_from_storage;

#[component]
pub fn AppShell(current_user: User) -> impl IntoView {
    let (current_page, set_current_page) = signal(Page::from_path(&current_path()));
    let initial_theme_mode = theme_mode_from_storage();

    let (theme_mode, set_theme_mode) = signal(initial_theme_mode);
    let (last_updated, set_last_updated) = signal(None::<String>);

    view! {
        <main class="app-shell">
            <AppHeader current_user=Some(current_user) />

            <div class="app-body">
                <SideMenu
                    current_page=current_page
                    set_current_page=set_current_page
                />

                <CurrentPage
                    current_page=current_page
                    last_updated=last_updated
                    set_last_updated=set_last_updated
                />
            </div>

            <SiteFooter
                theme_mode=theme_mode
                set_theme_mode=set_theme_mode
                last_updated=last_updated
            />
        </main>
    }
}
