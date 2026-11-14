use crate::frontend::components::menu_item::MenuItem;
use crate::frontend::routing::Page;
use leptos::prelude::*;

#[component]
pub fn SideMenu(
    current_page: ReadSignal<Page>,
    set_current_page: WriteSignal<Page>,
) -> impl IntoView {
    view! {
        <aside class="side-menu">
            <nav>
                <MenuItem label="Overview" target=Page::Overview current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Alerts" target=Page::Alerts current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Hosts" target=Page::Hosts current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Prometheus Targets" target=Page::Up current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Certificates" target=Page::Certificates current_page=current_page set_current_page=set_current_page />
                <MenuItem label="NixOS Generations" target=Page::Generations current_page=current_page set_current_page=set_current_page />
                <MenuItem label="Scheduled Tasks" target=Page::Tasks current_page=current_page set_current_page=set_current_page />
            </nav>
        </aside>
    }
}
