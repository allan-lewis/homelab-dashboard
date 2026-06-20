use crate::frontend::menu_state::save_menu_open;
use crate::frontend::routing::redirect_to;
use leptos::prelude::*;

#[component]
pub fn AppHeader(
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

