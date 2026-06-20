use crate::frontend::routing::redirect_to;
use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <main class="login-page">
            <div class="login-card">
                <h1>"Allan's Home Lab Dashboard"</h1>

                <p>
                    "Sign in with Authentik to continue."
                </p>

                <button
                    class="primary-button login-button"
                    on:click=move |_| redirect_to("/auth/login")
                >
                    <img
                        class="authentik-logo"
                        src="/authentik.png"
                        alt="Authentik"
                    />

                    <span>"Login with Authentik"</span>
                </button>
            </div>
        </main>
    }
}

