mod app;
mod tauri_api;
mod settings;
mod panel;
mod card;
mod button;
mod color;

use app::*;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
