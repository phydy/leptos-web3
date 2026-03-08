use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::privy::{
    privy_get_user,
    privy_init,
    privy_logout,
    privy_send_email_code,
    privy_verify_email_code,
    privy_connect_metamask,
    privy_connect_phantom,
    privy_disconnect_phantom,
    PrivyUser
};
use crate::auth::session::{
    clear_session,
    has_cached_session,
    save_session
};
//use crate::utils::metamask::MetaMask;
//use crate::utils::phantom::Phantom;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WalletType {
    MetaMask,
    Phantom,
}

// ---------------------------------------------------------------------------
// Public context type
// ---------------------------------------------------------------------------
#[derive(Clone, Copy)]
pub struct AuthContext {
    pub loading:        ReadSignal<bool>,
    pub authenticated:  ReadSignal<bool>,
    pub user:           ReadSignal<Option<PrivyUser>>,
    pub wallet_address: ReadSignal<Option<String>>,
    pub wallet_type:    ReadSignal<Option<WalletType>>,
    pub error:          ReadSignal<Option<String>>,
    pub email_otp_sent: ReadSignal<bool>,

    set_loading:        WriteSignal<bool>,
    set_authenticated:  WriteSignal<bool>,
    set_user:           WriteSignal<Option<PrivyUser>>,
    set_wallet_address: WriteSignal<Option<String>>,
    set_wallet_type:    WriteSignal<Option<WalletType>>,
    set_error:          WriteSignal<Option<String>>,
    set_email_otp_sent: WriteSignal<bool>,
}

impl AuthContext {
    // ------------------------------------------------------------------
    // Direct wallet connection
    // ------------------------------------------------------------------

    pub fn connect_metamask(self) {
        self.begin_loading();
        spawn_local(async move {
            match Self::do_connect_metamask().await {
                Ok(address) => self.on_wallet_connected(address, WalletType::MetaMask),
                Err(e)      => self.on_error(e),
            }
        });
    }

    async fn do_connect_metamask() -> Result<String, wasm_bindgen::JsValue> {
        privy_connect_metamask().await
    }

    pub fn connect_phantom(self) {
        self.begin_loading();
        spawn_local(async move {
            match Self::do_connect_phantom().await {
                Ok(address) => self.on_wallet_connected(address, WalletType::Phantom),
                Err(e)      => self.on_error(e),
            }
        });
    }

    async fn do_connect_phantom() -> Result<String, wasm_bindgen::JsValue> {
        privy_connect_phantom().await
    }

    // ------------------------------------------------------------------
    // Wallet disconnect
    // ------------------------------------------------------------------

    /// Disconnect the currently connected wallet.
    /// For Phantom this calls solana.disconnect(); MetaMask has no API so we
    /// just clear local state. The Privy email session (if any) is unaffected.
    pub fn disconnect_wallet(self) {
        let wtype = self.wallet_type.get_untracked();
        spawn_local(async move {
            if wtype == Some(WalletType::Phantom) {
                let _ = privy_disconnect_phantom().await;
            }
            // MetaMask: no programmatic disconnect API — clear state only.
            self.set_wallet_address.set(None);
            self.set_wallet_type.set(None);
            if self.user.get_untracked().is_none() {
                self.set_authenticated.set(false);
            }
        });
    }

    // ------------------------------------------------------------------
    // Privy email OTP
    // ------------------------------------------------------------------

    pub fn send_email_code(self, email: String) {
        self.begin_loading();
        spawn_local(async move {
            match privy_send_email_code(&email).await {
                Ok(_) => {
                    self.set_email_otp_sent.set(true);
                    self.set_loading.set(false);
                }
                Err(e) => self.on_error(e),
            }
        });
    }

    pub fn verify_email_code(self, email: String, code: String) {
        self.begin_loading();
        spawn_local(async move {
            match privy_verify_email_code(&email, &code).await {
                Ok(user) => {
                    self.set_email_otp_sent.set(false);
                    self.on_privy_login(user);
                }
                Err(e) => self.on_error(e),
            }
        });
    }

    pub fn reset_email_otp(self) {
        self.set_email_otp_sent.set(false);
        self.set_error.set(None);
    }

    // ------------------------------------------------------------------
    // Full logout — clears everything
    // ------------------------------------------------------------------

    pub fn logout(self) {
        spawn_local(async move {
            if self.wallet_type.get_untracked() == Some(WalletType::Phantom) {
                let _ = privy_disconnect_phantom().await;
            }
            let _ = privy_logout().await;
            clear_session();
            self.set_user.set(None);
            self.set_wallet_address.set(None);
            self.set_wallet_type.set(None);
            self.set_authenticated.set(false);
            self.set_email_otp_sent.set(false);
        });
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    fn begin_loading(self) {
        self.set_loading.set(true);
        self.set_error.set(None);
    }

    fn on_wallet_connected(self, address: String, wtype: WalletType) {
        self.set_wallet_address.set(Some(address));
        self.set_wallet_type.set(Some(wtype));
        self.set_authenticated.set(true);
        self.set_loading.set(false);
    }

    fn on_privy_login(self, user: PrivyUser) {
        save_session(&user.id);
        self.set_user.set(Some(user));
        self.set_authenticated.set(true);
        self.set_loading.set(false);
    }

    fn on_error(self, err: wasm_bindgen::JsValue) {
        let msg = err
            .as_string()
            .unwrap_or_else(|| "Operation failed — please try again.".into());
        self.set_error.set(Some(msg));
        self.set_loading.set(false);
    }
}

// ---------------------------------------------------------------------------
// Provider component
// ---------------------------------------------------------------------------

#[component]
pub fn AuthContextProvider(children: Children) -> impl IntoView {
    let (loading,          set_loading)          = signal(true);
    let (authenticated,    set_authenticated)    = signal(false);
    let (user,             set_user)             = signal(None::<PrivyUser>);
    let (wallet_address,   set_wallet_address)   = signal(None::<String>);
    let (wallet_type,      set_wallet_type)      = signal(None::<WalletType>);
    let (error,            set_error)            = signal(None::<String>);
    let (email_otp_sent,   set_email_otp_sent)   = signal(false);

    let ctx = AuthContext {
        loading, authenticated, user, wallet_address, wallet_type, error, email_otp_sent,
        set_loading, set_authenticated, set_user, set_wallet_address, set_wallet_type,
        set_error, set_email_otp_sent,
    };

    if has_cached_session() {
        set_authenticated.set(true);
    }

    spawn_local(async move {
        let app_id    = option_env!("PRIVY_APP_ID")    .unwrap_or("YOUR_PRIVY_APP_ID");
        let client_id = option_env!("PRIVY_CLIENT_ID") .unwrap_or("YOUR_PRIVY_CLIENT_ID");

        match privy_init(app_id, client_id).await {
            Ok(_) => {
                if let Some(u) = privy_get_user() {
                    save_session(&u.id);
                    set_authenticated.set(true);
                    set_user.set(Some(u));
                } else {
                    clear_session();
                    set_authenticated.set(false);
                }
            }
            Err(e) => {
                web_sys::console::error_1(&e);
            }
        }

        set_loading.set(false);
    });

    provide_context(ctx);
    children()
}

// ---------------------------------------------------------------------------
// Consumer hook
// ---------------------------------------------------------------------------

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect(
        "use_auth() called outside <AuthContextProvider>",
    )
}