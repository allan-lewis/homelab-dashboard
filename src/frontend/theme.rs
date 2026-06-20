#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

pub fn theme_mode_from_storage() -> ThemeMode {
    let Some(window) = web_sys::window() else {
        return ThemeMode::System;
    };

    let Ok(Some(storage)) = window.local_storage() else {
        return ThemeMode::System;
    };

    match storage.get_item("theme-mode").ok().flatten().as_deref() {
        Some("light") => ThemeMode::Light,
        Some("dark") => ThemeMode::Dark,
        _ => ThemeMode::System,
    }
}

fn theme_mode_to_storage_value(theme_mode: ThemeMode) -> &'static str {
    match theme_mode {
        ThemeMode::System => "system",
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    }
}

pub fn apply_theme_mode(theme_mode: ThemeMode) {
    let Some(window) = web_sys::window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    let Some(root) = document.document_element() else {
        return;
    };

    root.set_attribute("data-theme", theme_mode_to_storage_value(theme_mode))
        .expect("failed to set data-theme");

    if let Ok(Some(storage)) = window.local_storage() {
        storage
            .set_item("theme-mode", theme_mode_to_storage_value(theme_mode))
            .expect("failed to save theme mode");
    }
}
