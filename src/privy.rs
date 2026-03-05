use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

// ---------------------------------------------------------------------------
// JS extern bindings
//
// `module` tells wasm-bindgen to emit a static ES import:
//   import { init, loginWithMetaMask, … } from "/public/privy-bridge.js"
//
// The JS module system resolves this *before* any Rust code runs, so there
// is no window.* race condition and no readiness promise needed.
//
// The path must be absolute from the crate root (the leading "/" is required).
// ---------------------------------------------------------------------------
#[wasm_bindgen(module = "/public/privy.bundle.js")]
extern "C" {
    // init(appId: string, clientId: string) -> Promise<void>
    pub(crate) fn init(app_id: &str, client_id: &str) -> Promise;

    // loginWithMetaMask() -> Promise<JsValue>
    #[wasm_bindgen(js_name = "loginWithMetaMask")]
    pub(crate) fn login_with_metamask() -> Promise;

    // loginWithPhantom() -> Promise<JsValue>
    #[wasm_bindgen(js_name = "loginWithPhantom")]
    pub(crate) fn login_with_phantom() -> Promise;

    // logout() -> Promise<void>
    pub(crate) fn logout() -> Promise;

    // getUser() -> JsValue (serialised user or null)
    #[wasm_bindgen(js_name = "getUser")]
    pub(crate) fn get_user() -> JsValue;

    // isAuthenticated() -> bool
    #[wasm_bindgen(js_name = "isAuthenticated")]
    pub(crate) fn is_authenticated() -> bool;

    // getAccessToken() -> Promise<string | null>
    #[wasm_bindgen(js_name = "getAccessToken")]
    pub(crate) fn get_access_token() -> Promise;
}

// ---------------------------------------------------------------------------
// Rust types (deserialised from the JS bridge's serializeUser output)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrivyUser {
    pub id: String,
    pub wallet: Option<WalletInfo>,
    #[serde(rename = "linkedAccounts")]
    pub linked_accounts: Vec<serde_json::Value>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    #[serde(rename = "chainType")]
    pub chain_type: String,           // "ethereum" | "solana"
    #[serde(rename = "walletClient")]
    pub wallet_client: Option<String>, // "metamask" | "phantom"
}

// ---------------------------------------------------------------------------
// Async helpers
// ---------------------------------------------------------------------------

fn js_err_to_string(e: &JsValue) -> String {
    // e might be a JS Error object, a string primitive, or something else.
    // Try .message property first (Error objects), then fall back to toString().
    js_sys::Reflect::get(e, &JsValue::from_str("message"))
        .ok()
        .and_then(|v| v.as_string())
        .or_else(|| e.as_string())
        .unwrap_or_else(|| {
            js_sys::Object::from(e.clone())
                .to_string()
                .as_string()
                .unwrap_or_else(|| "unknown JS error".into())
        })
}

pub async fn privy_init(app_id: &str, client_id: &str) -> Result<(), JsValue> {
    JsFuture::from(init(app_id, client_id))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    Ok(())
}

pub async fn privy_login_metamask() -> Result<PrivyUser, JsValue> {
    let js_user = JsFuture::from(login_with_metamask())
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    serde_wasm_bindgen::from_value(js_user).map_err(|e| JsValue::from_str(&e.to_string()))
}

pub async fn privy_login_phantom() -> Result<PrivyUser, JsValue> {
    let js_user = JsFuture::from(login_with_phantom())
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    serde_wasm_bindgen::from_value(js_user).map_err(|e| JsValue::from_str(&e.to_string()))
}

pub async fn privy_logout() -> Result<(), JsValue> {
    JsFuture::from(logout())
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    Ok(())
}

pub fn privy_get_user() -> Option<PrivyUser> {
    let val = get_user();
    if val.is_null() || val.is_undefined() {
        return None;
    }
    serde_wasm_bindgen::from_value(val).ok()
}

pub async fn privy_get_access_token() -> Result<Option<String>, JsValue> {
    let val = JsFuture::from(get_access_token()).await?;
    if val.is_null() || val.is_undefined() {
        Ok(None)
    } else {
        Ok(val.as_string())
    }
}