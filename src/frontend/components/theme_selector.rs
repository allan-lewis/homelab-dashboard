use crate::frontend::theme::{apply_theme_mode, ThemeMode};
use leptos::prelude::*;

#[component]
pub fn ThemeSelector(
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

