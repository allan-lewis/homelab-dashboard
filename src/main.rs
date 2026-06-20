mod frontend;

use frontend::components::app_header::AppHeader;
use frontend::components::side_menu::SideMenu;
use frontend::menu_state::{menu_open_from_storage};
use frontend::models::{AuthState, User};
use frontend::pages::generations::GenerationsPage;
use frontend::pages::hosts::HostsPage;
use frontend::pages::loading::LoadingPage;
use frontend::pages::login::LoginPage;
use frontend::pages::overview::OverviewPage;
use frontend::pages::uptime::UptimePage;
use frontend::routing::{current_path, Page};
use frontend::theme::{apply_theme_mode, theme_mode_from_storage};
use gloo_net::http::Request;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[component]
fn CurrentPage(current_page: ReadSignal<Page>, name: String) -> impl IntoView {
    view! {
        {move || {
            match current_page.get() {
                Page::Overview => view! { <OverviewPage name=name.clone() /> }.into_any(),
                Page::Hosts => view! { <HostsPage /> }.into_any(),
                Page::Generations => view! { <GenerationsPage /> }.into_any(),
                Page::Uptime => view! { <UptimePage /> }.into_any(),
            }
        }}
    }
}

#[component]
fn AppShell(
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
                    name=name
                />
            </div>
        </main>
    }
}

#[component]
fn App() -> impl IntoView {
    let (auth_state, set_auth_state) = signal(AuthState::Loading);
    let (menu_open, set_menu_open) = signal(menu_open_from_storage());

    spawn_local(async move {
        let loaded_user = match Request::get("/api/me").send().await {
            Ok(response) => response.json::<Option<User>>().await.ok().flatten(),
            Err(_) => None,
        };

        match loaded_user {
            Some(user) => set_auth_state.set(AuthState::Authenticated(user)),
            None => set_auth_state.set(AuthState::Anonymous),
        }
    });

    view! {
        {move || {
            match auth_state.get() {
                AuthState::Loading => view! {
                    <LoadingPage />
                }
                .into_any(),

                AuthState::Anonymous => view! {
                    <LoginPage />
                }
                .into_any(),

                AuthState::Authenticated(current_user) => view! {
                    <AppShell
                        current_user=current_user
                        menu_open=menu_open
                        set_menu_open=set_menu_open
                    />
                }
                .into_any(),
            }
        }}
    }
}

fn main() {
    let initial_theme_mode = theme_mode_from_storage();
    apply_theme_mode(initial_theme_mode);

    leptos::mount::mount_to_body(App);
}
