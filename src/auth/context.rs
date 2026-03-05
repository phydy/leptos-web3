use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

//use super::{
use crate::privy::{privy_get_user, privy_init, privy_login_metamask, privy_login_phantom, privy_logout, PrivyUser};
use crate::auth::session::{clear_session, has_cached_session, save_session};
//};

// ---------------------------------------------------------------------------
// Public context type — cloneable handle passed through the tree
// ---------------------------------------------------------------------------
#[derive(Clone, Copy)]
pub struct AuthContext {
    /// True while Privy SDK is still initialising.
    pub loading: ReadSignal<bool>,
    /// True once the user is authenticated.
    pub authenticated: ReadSignal<bool>,
    /// The logged-in user, or None.
    pub user: ReadSignal<Option<PrivyUser>>,
    /// Non-fatal error string from the last login attempt.
    pub error: ReadSignal<Option<String>>,

    // Private write handles exposed only through action functions below
    set_loading: WriteSignal<bool>,
    set_authenticated: WriteSignal<bool>,
    set_user: WriteSignal<Option<PrivyUser>>,
    set_error: WriteSignal<Option<String>>,
}

impl AuthContext {
    /// Kick off MetaMask login (non-blocking — updates signals when done).
    pub fn login_metamask(self) {
        self.begin_login();
        spawn_local(async move {
            match privy_login_metamask().await {
                Ok(user) => self.on_login_success(user),
                Err(e) => self.on_login_error(e),
            }
        });
    }

    /// Kick off Phantom login.
    pub fn login_phantom(self) {
        self.begin_login();
        spawn_local(async move {
            match privy_login_phantom().await {
                Ok(user) => self.on_login_success(user),
                Err(e) => self.on_login_error(e),
            }
        });
    }

    pub fn logout(self) {
        spawn_local(async move {
            let _ = privy_logout().await;
            clear_session();
            self.set_user.set(None);
            self.set_authenticated.set(false);
        });
    }

    // ------------------------------------------------------------------
    // Helpers (private)
    // ------------------------------------------------------------------

    fn begin_login(self) {
        self.set_loading.set(true);
        self.set_error.set(None);
    }

    fn on_login_success(self, user: PrivyUser) {
        save_session(&user.id);
        self.set_authenticated.set(true);
        self.set_user.set(Some(user));
        self.set_loading.set(false);
    }

    fn on_login_error(self, err: wasm_bindgen::JsValue) {
        let msg = err
            .as_string()
            .unwrap_or_else(|| "Login failed — please try again.".into());
        self.set_error.set(Some(msg));
        self.set_loading.set(false);
    }
}

// ---------------------------------------------------------------------------
// Provider component
// ---------------------------------------------------------------------------

/// Wrap your entire app in this component once.
#[component]
pub fn AuthContextProvider(children: Children) -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (authenticated, set_authenticated) = signal(false);
    let (user, set_user) = signal(None::<PrivyUser>);
    let (error, set_error) = signal(None::<String>);

    let ctx = AuthContext {
        loading,
        authenticated,
        user,
        error,
        set_loading,
        set_authenticated,
        set_user,
        set_error,
    };

    // Optimistic UI — show as authenticated immediately if the browser cache
    // says so; Privy will re-validate the token asynchronously below.
    if has_cached_session() {
        set_authenticated.set(true);
    }

    // Initialise Privy on mount, then sync session state.
    // Replace "YOUR_PRIVY_APP_ID" with your real App ID from the Privy dashboard.
    spawn_local(async move {
        let app_id    = option_env!("PRIVY_APP_ID")     .unwrap_or("YOUR_PRIVY_APP_ID");
        let client_id = option_env!("PRIVY_CLIENT_ID")  .unwrap_or("YOUR_PRIVY_CLIENT_ID");

        match privy_init(app_id, client_id).await {
            Ok(_) => {
                // Privy is ready — check whether the stored token is still valid.
                if let Some(u) = privy_get_user() {
                    save_session(&u.id);
                    set_authenticated.set(true);
                    set_user.set(Some(u));
                } else {
                    // Token expired or no prior session.
                    clear_session();
                    set_authenticated.set(false);
                    set_user.set(None);
                }
            }
            Err(e) => {
                // Log the raw JS value so you can inspect it in DevTools
                // even if string extraction fails.
                web_sys::console::error_1(&e);
                leptos::logging::warn!(
                    "Privy init failed: {}",
                    js_sys::Reflect::get(&e, &wasm_bindgen::JsValue::from_str("message"))
                        .ok()
                        .and_then(|v| v.as_string())
                        .or_else(|| e.as_string())
                        .unwrap_or_else(|| "see console for JS error object".into())
                );
            }
        }

        set_loading.set(false);
    });

    provide_context(ctx);

    // Return children directly — no need to wrap in view! {}
    children()
}

// ---------------------------------------------------------------------------
// Consumer hook
// ---------------------------------------------------------------------------

/// Call this inside any component that needs auth state.
///
/// ```rust
/// let auth = use_auth();
/// let is_logged_in = move || auth.authenticated.get();
/// ```
pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect(
        "use_auth() called outside <AuthContextProvider> — \
         make sure AuthContextProvider wraps your router.",
    )
}
