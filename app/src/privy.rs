use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/public/privy.bundle.js")]
extern "C" {
    pub(crate) fn init(app_id: &str, client_id: &str) -> Promise;

    // Privy embedded wallet — address readers (sync)
    #[wasm_bindgen(js_name = "getPrivyEvmAddress")]
    pub(crate) fn get_privy_evm_address_js() -> JsValue;
    #[wasm_bindgen(js_name = "getPrivySolanaAddress")]
    pub(crate) fn get_privy_solana_address_js() -> JsValue;

    // Privy embedded wallet — EVM actions
    #[wasm_bindgen(js_name = "signMessagePrivyEvm")]
    pub(crate) fn sign_message_privy_evm_js(message: &str, address: &str) -> Promise;
    #[wasm_bindgen(js_name = "sendEthPrivy")]
    pub(crate) fn send_eth_privy_js(from: &str, to: &str, eth_amount: &str) -> Promise;
    #[wasm_bindgen(js_name = "sendERC20Privy")]
    pub(crate) fn send_erc20_privy_js(from: &str, token: &str, to: &str, amount: &str) -> Promise;
    #[wasm_bindgen(js_name = "callContractPrivy")]
    pub(crate) fn call_contract_privy_js(from: &str, contract: &str, data: &str) -> Promise;

    // Privy embedded wallet — Solana actions
    #[wasm_bindgen(js_name = "signMessagePrivySolana")]
    pub(crate) fn sign_message_privy_solana_js(message: &str) -> Promise;
    #[wasm_bindgen(js_name = "sendSolPrivy")]
    pub(crate) fn send_sol_privy_js(to: &str, sol_amount: &str) -> Promise;
    #[wasm_bindgen(js_name = "sendSPLTokenPrivy")]
    pub(crate) fn send_spl_token_privy_js(to: &str, mint: &str, amount: &str) -> Promise;
    #[wasm_bindgen(js_name = "invokeProgramPrivy")]
    pub(crate) fn invoke_program_privy_js(program_id: &str, data_hex: &str, accounts: &JsValue) -> Promise;

    // Wallet connect (gesture-safe, must stay in JS)
    #[wasm_bindgen(js_name = "connectMetaMask")]
    pub(crate) fn connect_metamask_js() -> Promise;
    #[wasm_bindgen(js_name = "connectPhantom")]
    pub(crate) fn connect_phantom_js() -> Promise;
    #[wasm_bindgen(js_name = "disconnectPhantom")]
    pub(crate) fn disconnect_phantom_js() -> Promise;

    // MetaMask wallet actions
    #[wasm_bindgen(js_name = "signMessageMetaMask")]
    pub(crate) fn sign_message_metamask_js(message: &str, address: &str) -> Promise;
    #[wasm_bindgen(js_name = "sendEthMetaMask")]
    pub(crate) fn send_eth_metamask_js(from: &str, to: &str, eth_amount: &str) -> Promise;
    #[wasm_bindgen(js_name = "sendERC20MetaMask")]
    pub(crate) fn send_erc20_metamask_js(from: &str, token: &str, to: &str, amount: &str) -> Promise;
    #[wasm_bindgen(js_name = "callContractMetaMask")]
    pub(crate) fn call_contract_metamask_js(from: &str, contract: &str, data: &str) -> Promise;

    // Phantom wallet actions
    #[wasm_bindgen(js_name = "signMessagePhantom")]
    pub(crate) fn sign_message_phantom_js(message: &str) -> Promise;
    #[wasm_bindgen(js_name = "sendSolPhantom")]
    pub(crate) fn send_sol_phantom_js(to: &str, sol_amount: &str) -> Promise;
    #[wasm_bindgen(js_name = "sendSPLTokenPhantom")]
    pub(crate) fn send_spl_token_phantom_js(to: &str, mint: &str, amount: &str) -> Promise;
    #[wasm_bindgen(js_name = "invokeProgramPhantom")]
    pub(crate) fn invoke_program_phantom_js(program_id: &str, data_hex: &str, accounts: &JsValue) -> Promise;

    // Counter Contract — EVM
    #[wasm_bindgen(js_name = "getCounterAddress")]
    pub(crate) fn get_counter_address_js() -> Promise;
    #[wasm_bindgen(js_name = "readCounterEVM")]
    pub(crate) fn read_counter_evm_js(contract_address: &str) -> Promise;
    #[wasm_bindgen(js_name = "counterIncMetaMask")]
    pub(crate) fn counter_inc_metamask_js(from: &str, contract_address: &str) -> Promise;
    #[wasm_bindgen(js_name = "counterIncByMetaMask")]
    pub(crate) fn counter_inc_by_metamask_js(from: &str, contract_address: &str, by: &str) -> Promise;
    #[wasm_bindgen(js_name = "counterIncPrivy")]
    pub(crate) fn counter_inc_privy_js(from: &str, contract_address: &str) -> Promise;
    #[wasm_bindgen(js_name = "counterIncByPrivy")]
    pub(crate) fn counter_inc_by_privy_js(from: &str, contract_address: &str, by: &str) -> Promise;

    // Counter Program — Solana (Anchor)
    #[wasm_bindgen(js_name = "readCounterSolana")]
    pub(crate) fn read_counter_solana_js(program_id: &str) -> Promise;
    #[wasm_bindgen(js_name = "counterInitializePhantom")]
    pub(crate) fn counter_initialize_phantom_js(program_id: &str) -> Promise;
    #[wasm_bindgen(js_name = "counterIncrementPhantom")]
    pub(crate) fn counter_increment_phantom_js(program_id: &str) -> Promise;
    #[wasm_bindgen(js_name = "counterInitializePrivy")]
    pub(crate) fn counter_initialize_privy_js(program_id: &str) -> Promise;
    #[wasm_bindgen(js_name = "counterIncrementPrivy")]
    pub(crate) fn counter_increment_privy_js(program_id: &str) -> Promise;

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

/// Connect MetaMask via JS (preserves user-gesture context for the popup).
pub async fn privy_connect_metamask() -> Result<String, JsValue> {
    let val = JsFuture::from(connect_metamask_js())
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string()
        .ok_or_else(|| JsValue::from_str("connectMetaMask: expected string address"))
}

/// Connect Phantom via JS (preserves user-gesture context for the popup).
pub async fn privy_connect_phantom() -> Result<String, JsValue> {
    let val = JsFuture::from(connect_phantom_js())
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string()
        .ok_or_else(|| JsValue::from_str("connectPhantom: expected string address"))
}

pub async fn privy_disconnect_phantom() -> Result<(), JsValue> {
    JsFuture::from(disconnect_phantom_js())
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// MetaMask wallet actions
// ---------------------------------------------------------------------------

pub async fn privy_sign_message_metamask(message: &str, address: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(sign_message_metamask_js(message, address))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("signMessageMetaMask: expected string"))
}

pub async fn privy_send_eth(from: &str, to: &str, eth_amount: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(send_eth_metamask_js(from, to, eth_amount))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("sendEthMetaMask: expected tx hash"))
}

pub async fn privy_send_erc20(from: &str, token: &str, to: &str, amount: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(send_erc20_metamask_js(from, token, to, amount))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("sendERC20MetaMask: expected tx hash"))
}

pub async fn privy_call_contract(from: &str, contract: &str, data: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(call_contract_metamask_js(from, contract, data))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("callContractMetaMask: expected tx hash"))
}

// ---------------------------------------------------------------------------
// Phantom wallet actions
// ---------------------------------------------------------------------------

pub async fn privy_sign_message_phantom(message: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(sign_message_phantom_js(message))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("signMessagePhantom: expected string"))
}

pub async fn privy_send_sol(to: &str, sol_amount: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(send_sol_phantom_js(to, sol_amount))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("sendSolPhantom: expected signature"))
}

pub async fn privy_send_spl_token(to: &str, mint: &str, amount: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(send_spl_token_phantom_js(to, mint, amount))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("sendSPLTokenPhantom: expected signature"))
}

pub async fn privy_invoke_program(program_id: &str, data_hex: &str, accounts: Vec<String>) -> Result<String, JsValue> {
    let accounts_js = serde_wasm_bindgen::to_value(&accounts)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let val = JsFuture::from(invoke_program_phantom_js(program_id, data_hex, &accounts_js))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("invokeProgramPhantom: expected signature"))
}

// ---------------------------------------------------------------------------
// Privy embedded wallet helpers
// ---------------------------------------------------------------------------

pub fn privy_get_evm_address() -> Option<String> {
    let v = get_privy_evm_address_js();
    if v.is_null() || v.is_undefined() { None } else { v.as_string() }
}

pub fn privy_get_solana_address() -> Option<String> {
    let v = get_privy_solana_address_js();
    if v.is_null() || v.is_undefined() { None } else { v.as_string() }
}

pub async fn privy_sign_message_evm(message: &str, address: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(sign_message_privy_evm_js(message, address))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("signMessagePrivyEvm: expected string"))
}

pub async fn privy_send_eth_privy(from: &str, to: &str, eth_amount: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(send_eth_privy_js(from, to, eth_amount))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("sendEthPrivy: expected tx hash"))
}

pub async fn privy_send_erc20_privy(from: &str, token: &str, to: &str, amount: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(send_erc20_privy_js(from, token, to, amount))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("sendERC20Privy: expected tx hash"))
}

pub async fn privy_call_contract_privy(from: &str, contract: &str, data: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(call_contract_privy_js(from, contract, data))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("callContractPrivy: expected tx hash"))
}

pub async fn privy_sign_message_solana(message: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(sign_message_privy_solana_js(message))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("signMessagePrivySolana: expected string"))
}

pub async fn privy_send_sol_privy(to: &str, sol_amount: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(send_sol_privy_js(to, sol_amount))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("sendSolPrivy: expected signature"))
}

pub async fn privy_send_spl_privy(to: &str, mint: &str, amount: &str) -> Result<String, JsValue> {
    let val = JsFuture::from(send_spl_token_privy_js(to, mint, amount))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("sendSPLTokenPrivy: expected signature"))
}

pub async fn privy_invoke_program_privy(program_id: &str, data_hex: &str, accounts: Vec<String>) -> Result<String, JsValue> {
    let accounts_js = serde_wasm_bindgen::to_value(&accounts)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let val = JsFuture::from(invoke_program_privy_js(program_id, data_hex, &accounts_js))
        .await
        .map_err(|e| JsValue::from_str(&js_err_to_string(&e)))?;
    val.as_string().ok_or_else(|| JsValue::from_str("invokeProgramPrivy: expected signature"))
}