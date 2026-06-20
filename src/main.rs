mod frontend;

use frontend::menu_state::{menu_open_from_storage, save_menu_open};
use frontend::models::{AuthState, HostState, HostStatus, User};
use frontend::pages::generations::GenerationsPage;
use frontend::pages::loading::LoadingPage;
use frontend::pages::login::LoginPage;
use frontend::pages::overview::OverviewPage;
use frontend::pages::uptime::UptimePage;
use frontend::routing::{current_path, push_path, redirect_to, Page};
use frontend::theme::{apply_theme_mode, theme_mode_from_storage, ThemeMode};
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[component]
fn AppHeader(
    name: String,
    email: String,
    menu_open: ReadSignal<bool>,
    set_menu_open: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <header class="app-header">
            <div class="header-left">
                <button
                    class="icon-button"
                    on:click=move |_| {
                        let new_value = !menu_open.get();

                        set_menu_open.set(new_value);
                        save_menu_open(new_value);
                    }
                >
                    "☰"
                </button>

                <h1>"Homelab Dashboard"</h1>
            </div>

            <div class="header-right">
                <span class="user-label">
                    {name}
                    " <"
                    {email}
                    ">"
                </span>

                <button
                    class="secondary-button"
                    on:click=move |_| redirect_to("/auth/logout")
                >
                    "Logout"
                </button>
            </div>
        </header>
    }
}

#[component]
fn MenuItem(
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

#[component]
fn ThemeSelector(
    theme_mode: ReadSignal<ThemeMode>,
    set_theme_mode: WriteSignal<ThemeMode>,
) -> impl IntoView {
    view! {
        <div class="theme-selector">
            <button
                class=move || if theme_mode.get() == ThemeMode::System { "theme-button active" } else { "theme-button" }
                on:click=move |_| {
                    set_theme_mode.set(ThemeMode::System);
                    apply_theme_mode(ThemeMode::System);
                }
                title="Follow system theme"
            >
                "AUTO"
            </button>

            <button
                class=move || if theme_mode.get() == ThemeMode::Light { "theme-button active" } else { "theme-button" }
                on:click=move |_| {
                    set_theme_mode.set(ThemeMode::Light);
                    apply_theme_mode(ThemeMode::Light);
                }
                title="Use light theme"
            >
                "LIGHT"
            </button>

            <button
                class=move || if theme_mode.get() == ThemeMode::Dark { "theme-button active" } else { "theme-button" }
                on:click=move |_| {
                    set_theme_mode.set(ThemeMode::Dark);
                    apply_theme_mode(ThemeMode::Dark);
                }
                title="Use dark theme"
            >
                "DARK"
            </button>
        </div>
    }
}

#[component]
fn SideMenu(
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

#[component]
fn HostsPage() -> impl IntoView {
    let (hosts, set_hosts) = signal(Vec::<HostStatus>::new());
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        loop {
            let loaded_hosts = match Request::get("/api/hosts").send().await {
                Ok(response) => response.json::<Vec<HostStatus>>().await.unwrap_or_default(),
                Err(_) => Vec::new(),
            };

            set_hosts.set(loaded_hosts);
            set_loaded.set(true);

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <section class="page-content">
            <h2>"Hosts"</h2>

            {move || {
                if !loaded.get() {
                    view! { <p>"Loading hosts..."</p> }.into_any()
                } else {
                    view! {
                        <table class="status-table">
                            <thead>
                                <tr>
                                <th>"Hostname"</th>
                                <th>"Persona"</th>
                                <th>"IP Address"</th>
                                <th>"Status"</th>
                                </tr>
                            </thead>

                            <tbody>
                                {hosts
                                    .get()
                                    .into_iter()
                                    .map(|host| {
                                        let status_class = match host.status {
                                            HostState::Up => "status-pill up",
                                            HostState::Down => "status-pill down",
                                            HostState::Unknown => "status-pill unknown",
                                        };

                                        let status_label = match host.status {
                                            HostState::Up => "Up",
                                            HostState::Down => "Down",
                                            HostState::Unknown => "Unknown",
                                        };
                                        view! {
                                            <tr>
                                                <td>{host.hostname}</td>
                                                <td>{host.persona}</td>
                                                <td>{host.ip_address}</td>
                                                <td>
                                                    <span class=status_class>
                                                        {status_label}
                                                    </span>
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }
            }}
        </section>
    }
}

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
