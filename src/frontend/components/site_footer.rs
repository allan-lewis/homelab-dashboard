use js_sys::Date;
use leptos::prelude::*;

use crate::frontend::components::theme_selector::ThemeSelector;
use crate::frontend::theme::ThemeMode;

fn current_year() -> i32 {
    Date::new_0().get_utc_full_year() as i32
}

#[component]
pub fn SiteFooter(
    theme_mode: ReadSignal<ThemeMode>,
    set_theme_mode: WriteSignal<ThemeMode>,
    #[prop(optional)] last_updated: Option<ReadSignal<Option<String>>>,
) -> impl IntoView {
    view! {
        <footer class="site-footer">
            <ThemeSelector theme_mode=theme_mode set_theme_mode=set_theme_mode />

            <div class="site-footer-center">
                <span>{format!("© {}", current_year())}</span>
                <span>"Allan's Home Lab"</span>
            </div>

            <p class="footer-last-updated">
                {move || match last_updated.and_then(|signal| signal.get()) {
                    Some(updated) => format!("Last updated {updated}"),
                    None => String::new(),
                }}
            </p>
        </footer>
    }
}
