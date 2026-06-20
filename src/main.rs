mod frontend;

use frontend::app::App;
use frontend::theme::{apply_theme_mode, theme_mode_from_storage};

fn main() {
    let initial_theme_mode = theme_mode_from_storage();
    apply_theme_mode(initial_theme_mode);

    leptos::mount::mount_to_body(App);
}
