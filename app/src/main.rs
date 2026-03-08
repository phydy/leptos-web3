use leptos::prelude::*;
use privy_wa::App;
use dotenv::dotenv;

fn main() {
    // set up logging
    dotenv().ok();

    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <App />
        }
    })
}
