use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/public/privy.bundle.js")]
extern "C" {
    pub(crate) fn init(app_id: &str, client_id: &str) -> Promise;

    pub(crate) fn logout() -> Promise;

    #[wasm_bindgen(js_name = "getUser")]
    pub(crate) fn get_user() -> JsValue;

    #[wasm_bindgen(js_name = "isAuthenticated")]
    pub(crate) fn is_authenticated() -> bool;

    #[wasm_bindgen(js_name = "getAccessToken")]
    pub(crate) fn get_access_token() -> JsValue;

    #[wasm_bindgen(js_name = "sendEmailCode")]
    pub(crate) fn send_email_code(email: &str) -> Promise;

    #[wasm_bindgen(js_name = "verifyEmailCode")]
    pub(crate) fn verify_email_code(email: &str, code: &str) -> Promise;
}

// ---------------------------------------------------------------------------
// Rust types
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrivyUser {
    pub id: String,
    pub wallet: Option<WalletInfo>,
    #[serde(rename = "linkedAccounts", default)]
    pub linked_accounts: Vec<serde_json::Value>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    #[serde(rename = "chainType")]
    pub chain_type: String,
    #[serde(rename = "walletClient")]
    pub wallet_client: Option<String>,
}

// ---------------------------------------------------------------------------
// Async helpers
// ---------------------------------------------------------------------------

fn js_err_to_string(e: &JsValue) -> String {
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

pub fn privy_get_access_token() -> Option<String> {
    let val = get_access_token();
    if val.is_null() || val.is_undefined() { None } else { val.as_string() }
}

pub async fn privy_send_email_code(email: &str) -> Result<(), JsValue> {
    JsFuture::from(send_email_code(email))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    Ok(())
}

pub async fn privy_verify_email_code(email: &str, code: &str) -> Result<PrivyUser, JsValue> {
    let js_user = JsFuture::from(verify_email_code(email, code))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    serde_wasm_bindgen::from_value(js_user).map_err(|e| JsValue::from_str(&e.to_string()))
}
