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
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="3" y="4" width="18" height="12" rx="1" />
                    <path d="M8 20h8M12 16v4" />
                </svg>
            </button>

            <button
                class=move || if theme_mode.get() == ThemeMode::Light { "theme-button active" } else { "theme-button" }
                on:click=move |_| {
                    set_theme_mode.set(ThemeMode::Light);
                    apply_theme_mode(ThemeMode::Light);
                }
                title="Use light theme"
            >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="4" />
                    <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
                </svg>
            </button>

            <button
                class=move || if theme_mode.get() == ThemeMode::Dark { "theme-button active" } else { "theme-button" }
                on:click=move |_| {
                    set_theme_mode.set(ThemeMode::Dark);
                    apply_theme_mode(ThemeMode::Dark);
                }
                title="Use dark theme"
            >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z" />
                </svg>
            </button>
        </div>
    }
}
