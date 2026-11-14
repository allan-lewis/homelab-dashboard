use leptos::prelude::*;

use crate::frontend::components::theme_selector::ThemeSelector;
use crate::frontend::theme::ThemeMode;

#[component]
pub fn SiteFooter(
    theme_mode: ReadSignal<ThemeMode>,
    set_theme_mode: WriteSignal<ThemeMode>,
    #[prop(optional)] last_updated: Option<ReadSignal<Option<String>>>,
) -> impl IntoView {
    view! {
        <footer class="site-footer">
            <ThemeSelector theme_mode=theme_mode set_theme_mode=set_theme_mode />

            <p class="footer-last-updated">
                {move || match last_updated.and_then(|signal| signal.get()) {
                    Some(updated) => format!("Last updated {updated}"),
                    None => String::new(),
                }}
            </p>
        </footer>
    }
}
