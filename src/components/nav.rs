use leptos::prelude::*;
use crate::auth::context::{use_auth, WalletType};

fn truncate_address(addr: &str) -> String {
    let chars: Vec<char> = addr.chars().collect();
    if chars.len() >= 10 {
        format!(
            "{}…{}",
            chars[..6].iter().collect::<String>(),
            chars[chars.len() - 4..].iter().collect::<String>()
        )
    } else {
        addr.to_string()
    }
}

#[component]
pub fn Nav() -> impl IntoView {
    let auth = use_auth();
    let on_logout          = move |_| auth.logout();
    let on_disconnect_wallet = move |_| auth.disconnect_wallet();

    view! {
        <nav class="top-nav">
            <a href="/" class="nav-brand">"🔐 App"</a>

            <div class="nav-actions">
                <Show
                    when=move || auth.authenticated.get()
                    fallback=|| view! { <a href="/" class="nav-link">"Login"</a> }
                >
                    // Wallet badge — shown when a wallet is connected
                    <Show when=move || auth.wallet_address.get().is_some()>
                        {move || {
                            let addr  = auth.wallet_address.get().unwrap_or_default();
                            let label = match auth.wallet_type.get() {
                                Some(WalletType::MetaMask) => "🦊",
                                Some(WalletType::Phantom)  => "👻",
                                None                       => "🔑",
                            };
                            view! {
                                <span class="wallet-address">
                                    {label}" "{truncate_address(&addr)}
                                </span>
                                <button
                                    class="nav-btn disconnect-wallet"
                                    on:click=on_disconnect_wallet
                                    title="Disconnect wallet"
                                >
                                    "Disconnect wallet"
                                </button>
                            }
                        }}
                    </Show>

                    // Email badge — shown when signed in via Privy email
                    <Show when=move || auth.user.get().is_some()>
                        {move || auth.user.get().map(|u| {
                            let email = u.linked_accounts.iter().find_map(|a| {
                                a.get("address").and_then(|v| v.as_str()).map(|s| s.to_string())
                            }).unwrap_or_else(|| truncate_address(&u.id));
                            view! {
                                <span class="wallet-address">"✉️ "{email}</span>
                            }
                        })}
                    </Show>

                    <a href="/dashboard" class="nav-link">"Dashboard"</a>
                    <button class="nav-btn logout" on:click=on_logout>
                        "Sign out"
                    </button>
                </Show>
            </div>
        </nav>
    }
}