use leptos::{component, view, prelude::*};
use crate::auth::context::use_auth;
use crate::components::login::LoginPanel;

#[component]
pub fn HomePage() -> impl IntoView {
    let auth = use_auth();

    view! {
        <div class="page home-page">
            // If already authenticated, skip straight to dashboard
            <Show
                when=move || auth.authenticated.get()
                fallback=|| view! {
                    <section class="hero">
                        <h1>"Welcome"</h1>
                        <p>"Connect your wallet to continue."</p>
                        <LoginPanel />
                    </section>
                }
            >
                <section class="hero">
                    <h1>"You're in!"</h1>
                    <p>"Head to your dashboard."</p>
                    <a href="/dashboard" class="btn primary">"Go to Dashboard →"</a>
                </section>
            </Show>
        </div>
    }
}