use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;
use js_sys::{Reflect, Function};

pub struct Phantom {
    pub solana: JsValue,
}

impl Phantom {

    pub fn new() -> Result<Self, JsValue> {
        let window = window().unwrap();
        let solana = Reflect::get(&window, &"solana".into())
            .unwrap_or(JsValue::UNDEFINED);

        if solana.is_null() || solana.is_undefined() {
            return Err(JsValue::from_str("Phantom not installed"));
        }

        let is_phantom = Reflect::get(&solana, &"isPhantom".into())
            .map(|v| v.is_truthy())
            .unwrap_or(false);
        if !is_phantom {
            return Err(JsValue::from_str("Phantom not found — another Solana wallet may be active"));
        }

        Ok(Self { solana })
    }

    pub async fn disconnect(&self) -> Result<(), JsValue> {
        let disconnect_fn = Reflect::get(&self.solana, &"disconnect".into())?
            .dyn_into::<Function>()?;
        let res = disconnect_fn.call0(&self.solana)?.dyn_into::<js_sys::Promise>()?;
        wasm_bindgen_futures::JsFuture::from(res).await?;
        Ok(())
    }

    pub async fn connect(&self) -> Result<JsValue, JsValue> {
        let connect = Reflect::get(&self.solana, &"connect".into())?
            .dyn_into::<Function>()?;
        let res = connect.call0(&self.solana)?
            .dyn_into::<js_sys::Promise>()?;
        wasm_bindgen_futures::JsFuture::from(res).await
    }

    pub fn public_key(&self) -> Result<String, JsValue> {
        let pk = Reflect::get(&self.solana, &"publicKey".into())?;
        if pk.is_null() || pk.is_undefined() {
            return Err(JsValue::from_str("Phantom: no public key — call connect() first"));
        }
        let to_string_fn = Reflect::get(&pk, &"toString".into())?
            .dyn_into::<Function>()?;
        to_string_fn
            .call0(&pk)?
            .as_string()
            .ok_or_else(|| JsValue::from_str("Phantom: publicKey.toString() failed"))
    }

    pub async fn sign_message(&self, message: &str) -> Result<JsValue, JsValue> {
        let sign_fn = Reflect::get(&self.solana, &"signMessage".into())?
            .dyn_into::<Function>()?;
        let msg = js_sys::Uint8Array::from(message.as_bytes());
        let res = sign_fn
            .call1(&self.solana, &msg.into())?
            .dyn_into::<js_sys::Promise>()?;
        wasm_bindgen_futures::JsFuture::from(res).await
    }

    pub async fn send_transaction(
        &self,
        tx: JsValue
    ) -> Result<JsValue, JsValue> {
        let fn_send = Reflect::get(
            &self.solana,
            &"signAndSendTransaction".into(),
        )?
        .dyn_into::<Function>()?;
        let res = fn_send
            .call1(&self.solana, &tx)?
            .dyn_into::<js_sys::Promise>()?;
        wasm_bindgen_futures::JsFuture::from(res).await
    }
}