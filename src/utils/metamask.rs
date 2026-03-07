use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;
use js_sys::{Reflect, Function, Object};


pub struct MetaMask {
    pub ethereum: JsValue,
    pub request: Function
}


impl MetaMask {
    pub fn new() -> Result<Self, JsValue> {
        let window = window().expect("no global window");
        let ethereum = Reflect::get(&window, &JsValue::from_str("ethereum"))
            .unwrap_or(JsValue::UNDEFINED);

        if ethereum.is_null() || ethereum.is_undefined() {
            return Err(JsValue::from_str("MetaMask not installed"));
        }

        let is_metamask = Reflect::get(&ethereum, &JsValue::from_str("isMetaMask"))
            .map(|v| v.is_truthy())
            .unwrap_or(false);
        if !is_metamask {
            return Err(JsValue::from_str("MetaMask not found — another wallet may be active"));
        }

        let request = Reflect::get(&ethereum, &JsValue::from_str("request"))
            .unwrap()
            .dyn_into::<Function>()
            .unwrap();
        Ok(Self { ethereum, request })
    }

    pub async fn request_accounts(&self) -> Result<js_sys::Array, JsValue> {
        let params = Object::new();
        Reflect::set(&params, &JsValue::from_str("method"), &JsValue::from_str("eth_requestAccounts"))?;
        let res = self.request.call1(&self.ethereum, &params)?.dyn_into::<js_sys::Promise>()?;
        let val = wasm_bindgen_futures::JsFuture::from(res).await?;
        val.dyn_into::<js_sys::Array>()
            .map_err(|_| JsValue::from_str("eth_requestAccounts: unexpected response type"))
    }

    pub async fn sign_message(
        &self,
        message: &str,
        address: &str,
    ) -> Result<JsValue, JsValue> {
        let hex_message = format!(
            "0x{}",
            message.bytes().map(|b| format!("{:02x}", b)).collect::<String>()
        );

        let params = Object::new();
        Reflect::set(
            &params,
            &JsValue::from_str("method"),
            &JsValue::from_str("personal_sign"),
        )?;
        let args = js_sys::Array::new();
        args.push(&JsValue::from_str(&hex_message));
        args.push(&JsValue::from_str(&address.to_lowercase()));
        Reflect::set(&params, &JsValue::from_str("params"), &args)?;
        let res = self.request
            .call1(&self.ethereum, &params)?
            .dyn_into::<js_sys::Promise>()?;
        wasm_bindgen_futures::JsFuture::from(res).await
    }

    pub async fn eth_accounts(&self) -> Result<js_sys::Array, JsValue> {
        let params = Object::new();
        Reflect::set(&params, &JsValue::from_str("method"), &JsValue::from_str("eth_accounts"))?;
        let res = self.request.call1(&self.ethereum, &params)?.dyn_into::<js_sys::Promise>()?;
        let val = wasm_bindgen_futures::JsFuture::from(res).await?;
        Ok(val.dyn_into::<js_sys::Array>()?)
    }

    pub async fn eth_chain_id(&self) -> Result<u64, JsValue> {
        let params = Object::new();
        Reflect::set(&params, &JsValue::from_str("method"), &JsValue::from_str("eth_chainId"))?;
        let res = self.request.call1(&self.ethereum, &params)?.dyn_into::<js_sys::Promise>()?;
        let val = wasm_bindgen_futures::JsFuture::from(res).await?;
        let hex = val.as_string().ok_or_else(|| JsValue::from_str("eth_chainId: not a string"))?;
        u64::from_str_radix(hex.trim_start_matches("0x"), 16)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn write_contract(
        &self,
        from: &str,
        to: &str,
        data: &str,
    ) -> Result<JsValue, JsValue> {

        let tx = Object::new();

        Reflect::set(&tx, &"from".into(), &from.into())?;
        Reflect::set(&tx, &"to".into(), &to.into())?;
        Reflect::set(&tx, &"data".into(), &data.into())?;

        let txs = js_sys::Array::new();
        txs.push(&tx);

        let params = Object::new();

        Reflect::set(
            &params,
            &"method".into(),
            &"eth_sendTransaction".into(),
        )?;

        Reflect::set(&params, &"params".into(), &txs)?;

        let res = self.request
            .call1(&self.ethereum, &params)?
            .dyn_into::<js_sys::Promise>()?;

        wasm_bindgen_futures::JsFuture::from(res).await
    }
}