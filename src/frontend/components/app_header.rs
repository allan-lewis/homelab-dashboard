use crate::frontend::models::User;
use crate::frontend::routing::redirect_to;
use leptos::prelude::*;

#[component]
pub fn AppHeader(current_user: Option<User>) -> impl IntoView {
    view! {
        <header class="site-header">
            <div class="header-left">
                <img
                    class="site-header-favicon site-header-favicon-light"
                    src="/header-icon-light.svg"
                    alt=""
                />
                <img
                    class="site-header-favicon site-header-favicon-dark"
                    src="/header-icon-dark.svg"
                    alt=""
                />
                <h1>"Allan's Home Lab Dashboard"</h1>
            </div>

            <div class="header-right">
                {match current_user {
                    Some(user) => {
                        view! {
                            <span class="user-label">
                                {user.name}
                                " <"
                                {user.email}
                                ">"
                            </span>

                            <button
                                class="secondary-button"
                                on:click=move |_| redirect_to("/auth/logout")
                            >
                                "Logout"
                            </button>
                        }
                            .into_any()
                    }
                    None => {
                        view! { <a class="secondary-button" href="/auth/login">"Login"</a> }
                            .into_any()
                    }
                }}
            </div>
        </header>
    }
}
