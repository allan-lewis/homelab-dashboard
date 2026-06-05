use gloo_net::http::Request;
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Debug, Deserialize)]
struct User {
    name: String,
    email: String,
}

#[derive(Clone, Debug)]
enum AuthState {
    Loading,
    Anonymous,
    Authenticated(User),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Overview,
    Hosts,
    Generations,
    Uptime,
}

fn redirect_to(path: &str) {
    let window = web_sys::window().expect("missing browser window");

    window
        .location()
        .set_href(path)
        .expect("failed to redirect");
}

#[component]
fn LoadingPage() -> impl IntoView {
    view! {
        <main class="login-page">
            <p>"Loading..."</p>
        </main>
    }
}

#[component]
fn LoginPage() -> impl IntoView {
    view! {
        <main class="login-page">
            <button
                class="primary-button"
                on:click=move |_| redirect_to("/auth/login")
            >
                "Login"
            </button>
        </main>
    }
}

#[component]
fn AppHeader(
    name: String,
    email: String,
    set_menu_open: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <header class="app-header">
            <div class="header-left">
                <button
                    class="icon-button"
                    on:click=move |_| {
                        set_menu_open.update(|open| *open = !*open);
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
            }
        >
            {label}
        </button>
    }
}

#[component]
fn SideMenu(
    menu_open: ReadSignal<bool>,
    current_page: ReadSignal<Page>,
    set_current_page: WriteSignal<Page>,
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
        </aside>
    }
}

#[component]
fn OverviewPage(name: String) -> impl IntoView {
    view! {
        <section class="page-content">
            <h2>"Overview"</h2>
            <p>"Welcome " {name}</p>
            <p>"This will show high-level fleet status: host count, unhealthy hosts, stale check-ins, and recent changes."</p>
        </section>
    }
}

#[component]
fn HostsPage() -> impl IntoView {
    view! {
        <section class="page-content">
            <h2>"Hosts"</h2>
            <p>"This page will list each host with status, IP, role, OS, and last-seen timestamp."</p>
        </section>
    }
}

#[component]
fn GenerationsPage() -> impl IntoView {
    view! {
        <section class="page-content">
            <h2>"NixOS Generations"</h2>
            <p>"This page will compare booted/current NixOS generations across hosts and flag mismatches."</p>
        </section>
    }
}

#[component]
fn UptimePage() -> impl IntoView {
    view! {
        <section class="page-content">
            <h2>"Uptime"</h2>
            <p>"This page will show uptime, reboot history, and hosts that may need attention after upgrades."</p>
        </section>
    }
}

#[component]
fn CurrentPage(
    current_page: ReadSignal<Page>,
    name: String,
) -> impl IntoView {
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
    let (current_page, set_current_page) = signal(Page::Overview);

    view! {
        <main class="app-shell">
            <AppHeader
                name=name.clone()
                email=email
                set_menu_open=set_menu_open
            />

            <div class="app-body">
                <SideMenu
                    menu_open=menu_open
                    current_page=current_page
                    set_current_page=set_current_page
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
    let (menu_open, set_menu_open) = signal(true);

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
    leptos::mount::mount_to_body(App);
}