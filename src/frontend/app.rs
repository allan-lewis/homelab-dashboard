use gloo_net::http::Request;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::app_shell::AppShell;
use crate::frontend::menu_state::menu_open_from_storage;
use crate::frontend::models::{AuthState, User};
use crate::frontend::pages::loading::LoadingPage;
use crate::frontend::pages::login::LoginPage;

#[component]
pub fn App() -> impl IntoView {
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

