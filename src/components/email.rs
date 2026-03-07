use leptos::prelude::*;
use crate::auth::context::use_auth;

#[component]
pub fn EmailLoginPanel() -> impl IntoView {
    let auth = use_auth();

    let (email, set_email) = signal(String::new());
    let (code, set_code) = signal(String::new());

    let on_send = move |_| {
        let e = email.get();
        if !e.is_empty() {
            auth.send_email_code(e);
        }
    };

    let on_verify = move |_| {
        let e = email.get();
        let c = code.get();
        if !e.is_empty() && !c.is_empty() {
            auth.verify_email_code(e, c);
        }
    };

    let on_change_email = move |_| {
        auth.reset_email_otp();
        set_code.set(String::new());
    };

    view! {
        <div class="email-login-panel">
            <Show
                when=move || !auth.email_otp_sent.get()
                fallback=move || view! {
                    <p class="otp-hint">
                        "We sent a 6-digit code to "
                        <strong>{email.get()}</strong>
                    </p>
                    <div class="input-row">
                        <input
                            type="text"
                            inputmode="numeric"
                            autocomplete="one-time-code"
                            maxlength="6"
                            placeholder="123456"
                            class="otp-input"
                            prop:value=move || code.get()
                            on:input=move |ev| set_code.set(event_target_value(&ev))
                            disabled=move || auth.loading.get()
                        />
                        <button
                            class="btn primary"
                            on:click=on_verify
                            disabled=move || auth.loading.get() || code.get().len() < 6
                        >
                            {move || if auth.loading.get() { "Verifying…" } else { "Verify" }}
                        </button>
                    </div>
                    <button class="link-btn" on:click=on_change_email>
                        "Use a different email"
                    </button>
                }
            >
                <div class="input-row">
                    <input
                        type="email"
                        autocomplete="email"
                        placeholder="you@example.com"
                        class="email-input"
                        prop:value=move || email.get()
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                        disabled=move || auth.loading.get()
                    />
                    <button
                        class="btn primary"
                        on:click=on_send
                        disabled=move || auth.loading.get() || email.get().is_empty()
                    >
                        {move || if auth.loading.get() { "Sending…" } else { "Send code" }}
                    </button>
                </div>
            </Show>
        </div>
    }
}