pub fn menu_open_from_storage() -> bool {
    let Some(window) = web_sys::window() else {
        return true;
    };

    let Ok(Some(storage)) = window.local_storage() else {
        return true;
    };

    match storage.get_item("menu-open").ok().flatten().as_deref() {
        Some("false") => false,
        _ => true,
    }
}

pub fn save_menu_open(menu_open: bool) {
    let Some(window) = web_sys::window() else {
        return;
    };

    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };

    let _ = storage.set_item("menu-open", if menu_open { "true" } else { "false" });
}
