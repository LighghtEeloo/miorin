mod app;
mod settings;
mod tauri_api;

mod ui {
    pub mod panel;
    pub mod card;
    pub mod button;
    pub mod color;
    pub mod toggle;
    pub mod filter;
}

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
