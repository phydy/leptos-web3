use leptos::prelude::*;
use leptos_router::*;
use crate::auth::context::use_auth;

#[component]
pub fn Nav() -> impl IntoView {
    let auth = use_auth();
    let on_logout = move |_| auth.logout();

    view! {
        <nav class="top-nav">
            <a href="/" class="nav-brand">"🔐 Privy App"</a>

            <div class="nav-actions">
                <Show
                    when=move || auth.authenticated.get()
                    fallback=|| view! { <a href="/" class="nav-link">"Login"</a> }
                >
                    // Show truncated wallet address
                    {move || {
                        auth.user.get()
                            .and_then(|u| u.wallet)
                            .map(|w| {
                                let addr = w.address;
                                // Safe truncation — wallet addresses are ASCII
                                // but guard anyway so a short/malformed value
                                // doesn't panic.
                                let chars: Vec<char> = addr.chars().collect();
                                let short = if chars.len() >= 10 {
                                    format!(
                                        "{}…{}",
                                        chars[..6].iter().collect::<String>(),
                                        chars[chars.len()-4..].iter().collect::<String>()
                                    )
                                } else {
                                    addr.clone()
                                };
                                view! {
                                    <span class="wallet-address">{short}</span>
                                }
                            })
                    }}
                    <a href="/dashboard" class="nav-link">"Dashboard"</a>
                    <button class="nav-btn logout" on:click=on_logout>
                        "Disconnect"
                    </button>
                </Show>
            </div>
        </nav>
    }
}