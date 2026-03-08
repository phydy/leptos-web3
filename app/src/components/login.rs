use leptos::{component, view, prelude::*};
use crate::auth::context::use_auth;
use crate::components::email::EmailLoginPanel;

#[component]
pub fn LoginPanel() -> impl IntoView {
    let auth = use_auth();

    let on_metamask = move |_| auth.connect_metamask();
    let on_phantom  = move |_| auth.connect_phantom();

    view! {
        <div class="login-panel">
            <h2>"Sign in"</h2>

            <Show when=move || auth.error.get().is_some()>
                <div class="error-banner">
                    {move || auth.error.get().unwrap_or_default()}
                </div>
            </Show>

            <EmailLoginPanel />

            <div class="or-divider">
                <span>"or connect a wallet"</span>
            </div>

            <div class="wallet-buttons">
                <button
                    class="wallet-btn metamask"
                    on:click=on_metamask
                    disabled=move || auth.loading.get()
                >
                    <img src="/public/icons/metamask.svg" alt="MetaMask" />
                    "Connect MetaMask"
                </button>
                <button
                    class="wallet-btn phantom"
                    on:click=on_phantom
                    disabled=move || auth.loading.get()
                >
                    <img src="/public/icons/phantom.svg" alt="Phantom" />
                    "Connect Phantom"
                </button>
            </div>

            <Show when=move || auth.loading.get()>
                <p class="loading-hint">"Connecting…"</p>
            </Show>
        </div>
    }
}