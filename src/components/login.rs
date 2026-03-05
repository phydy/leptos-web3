use leptos::{component, view, prelude::*};
use crate::auth::context::use_auth;

/// Drop this component anywhere you want a login UI.
/// Shows MetaMask + Phantom buttons, a loading spinner, and an error banner.
#[component]
pub fn LoginPanel() -> impl IntoView {
    let auth = use_auth();

    let on_metamask = move |_| auth.login_metamask();
    let on_phantom  = move |_| auth.login_phantom();

    view! {
        <div class="login-panel">
            <h2>"Connect your wallet"</h2>

            // Error banner
            <Show when=move || auth.error.get().is_some()>
                <div class="error-banner">
                    {move || auth.error.get().unwrap_or_default()}
                </div>
            </Show>

            <div class="wallet-buttons">
                // MetaMask
                <button
                    class="wallet-btn metamask"
                    on:click=on_metamask
                    disabled=move || auth.loading.get()
                >
                    <img src="/public/favicon.ico" alt="MetaMask logo" />
                    "MetaMask"
                </button>

                // Phantom
                <button
                    class="wallet-btn phantom"
                    on:click=on_phantom
                    disabled=move || auth.loading.get()
                >
                    <img src="/public/favicon.ico" alt="Phantom logo" />
                    "Phantom"
                </button>
            </div>

            <Show when=move || auth.loading.get()>
                <p class="loading-hint">"Waiting for wallet…"</p>
            </Show>
        </div>
    }
}