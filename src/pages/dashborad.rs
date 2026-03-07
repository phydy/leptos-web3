use leptos::prelude::*;
use crate::auth::context::use_auth;
use crate::components::protect::RequireAuth;

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <RequireAuth>
            <DashboardInner />
        </RequireAuth>
    }
}

#[component]
fn DashboardInner() -> impl IntoView {
    let auth = use_auth();

    view! {
        <div class="page dashboard-page">
            <h1>"Dashboard"</h1>

            {move || auth.user.get().map(|user| {
                let wallet_info = user.wallet.clone();

                view! {
                    <div class="user-card">
                        <h2>"Session"</h2>
                        <dl>
                            <dt>"Privy ID"</dt>
                            <dd>{user.id.clone()}</dd>

                            {wallet_info.map(|w| view! {
                                <>
                                    <dt>"Wallet"</dt>
                                    <dd>{w.address}</dd>
                                    <dt>"Chain"</dt>
                                    <dd>{w.chain_type}</dd>
                                    <dt>"Client"</dt>
                                    <dd>{w.wallet_client.unwrap_or_else(|| "—".into())}</dd>
                                </>
                            })}
                        </dl>
                    </div>
                }
            })}
        </div>
    }
}