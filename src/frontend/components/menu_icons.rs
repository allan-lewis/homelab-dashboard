use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuIcon {
    Overview,
    Slo,
    Alerts,
    Hosts,
    Targets,
    Certificates,
    Generations,
    Tasks,
}

impl MenuIcon {
    pub fn view(self) -> AnyView {
        match self {
            MenuIcon::Overview => {
                view! {
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <rect x="3" y="3" width="8" height="8" rx="1.5" />
                        <rect x="13" y="3" width="8" height="8" rx="1.5" />
                        <rect x="3" y="13" width="8" height="8" rx="1.5" />
                        <rect x="13" y="13" width="8" height="8" rx="1.5" />
                    </svg>
                }
                    .into_any()
            }
            MenuIcon::Slo => {
                view! {
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M9 2h6" />
                        <path d="M12 2v3" />
                        <circle cx="12" cy="13" r="8" />
                        <path d="M12 13l3-3" />
                    </svg>
                }
                    .into_any()
            }
            MenuIcon::Alerts => {
                view! {
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M18 16v-5a6 6 0 1 0-12 0v5l-2 2h16l-2-2Z" />
                        <path d="M10 20a2 2 0 0 0 4 0" />
                    </svg>
                }
                    .into_any()
            }
            MenuIcon::Hosts => {
                view! {
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <rect x="3" y="4" width="18" height="6" rx="1.5" />
                        <rect x="3" y="14" width="18" height="6" rx="1.5" />
                        <path d="M7 7h.01M7 17h.01" />
                    </svg>
                }
                    .into_any()
            }
            MenuIcon::Targets => {
                view! {
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="12" cy="12" r="8" />
                        <circle cx="12" cy="12" r="3" />
                        <path d="M12 2v3M12 19v3M2 12h3M19 12h3" />
                    </svg>
                }
                    .into_any()
            }
            MenuIcon::Certificates => {
                view! {
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6l7-3Z" />
                        <path d="M9 12l2 2 4-4" />
                    </svg>
                }
                    .into_any()
            }
            MenuIcon::Generations => {
                view! {
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 3l9 5-9 5-9-5 9-5Z" />
                        <path d="M3 13l9 5 9-5" />
                    </svg>
                }
                    .into_any()
            }
            MenuIcon::Tasks => {
                view! {
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <rect x="5" y="4" width="14" height="17" rx="2" />
                        <path d="M9 3h6a1 1 0 0 1 1 1v1H8V4a1 1 0 0 1 1-1Z" />
                        <path d="M9 13l2 2 4-4" />
                    </svg>
                }
                    .into_any()
            }
        }
    }
}
