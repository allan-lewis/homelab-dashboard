use leptos::wasm_bindgen::JsValue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Overview,
    Alerts,
    Certificates,
    Hosts,
    Generations,
    Tasks,
    Uptime,
}

pub fn redirect_to(path: &str) {
    let window = web_sys::window().expect("missing browser window");

    window
        .location()
        .set_href(path)
        .expect("failed to redirect");
}

impl Page {
    pub fn path(self) -> &'static str {
        match self {
            Page::Overview => "/",
            Page::Alerts => "/alerts",
            Page::Certificates => "/certificates",
            Page::Hosts => "/hosts",
            Page::Generations => "/generations",
            Page::Tasks => "/tasks",
            Page::Uptime => "/uptime",
        }
    }

    pub fn from_path(path: &str) -> Self {
        match path {
            "/alerts" => Page::Alerts,
            "/certificates" => Page::Certificates,
            "/hosts" => Page::Hosts,
            "/generations" => Page::Generations,
            "/tasks" => Page::Tasks,
            "/uptime" => Page::Uptime,
            _ => Page::Overview,
        }
    }
}

pub fn current_path() -> String {
    web_sys::window()
        .expect("missing window")
        .location()
        .pathname()
        .unwrap_or_else(|_| "/".to_string())
}

pub fn push_path(path: &str) {
    let window = web_sys::window().expect("missing window");
    window
        .history()
        .expect("missing history")
        .push_state_with_url(&JsValue::NULL, "", Some(path))
        .expect("failed to push history state");
}
