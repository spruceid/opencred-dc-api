use async_trait::async_trait;
use dc_api_core::openid4vp::verifier::request_signer::{JWK, RequestSigner};
use js_sys::{Function, Promise, Reflect, Uint8Array};
use std::{fmt::Debug, sync::Arc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(typescript_custom_section)]
const TS_IFACE: &'static str = r#"
export interface RequestSigner {
  // Keep these synchronous to match the Rust trait above.
  alg(): string;
  jwk(): any;

  // Must return a Promise of bytes.
  sign(payload: Uint8Array): Promise<Uint8Array>;

  // Optional; if absent, Rust falls back to sign()
  try_sign?(payload: Uint8Array): Promise<Uint8Array>;
}
"#;

#[derive(Debug, Clone)]
pub struct JsErr(String);

impl std::fmt::Display for JsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<JsValue> for JsErr {
    fn from(e: JsValue) -> Self {
        // best effort stringify
        let s = js_sys::JSON::stringify(&e)
            .ok()
            .and_then(|js| js.as_string())
            .or_else(|| e.as_string())
            .unwrap_or_else(|| "JS error".to_string());
        JsErr(s)
    }
}

fn call_method(obj: &JsValue, name: &str, args: &[JsValue]) -> Result<JsValue, JsErr> {
    let method = Reflect::get(obj, &JsValue::from_str(name)).map_err(JsErr::from)?;
    if !method.is_function() {
        return Err(JsErr(format!("Expected method `{name}` to be a function")));
    }
    let f: &Function = method.unchecked_ref();
    f.call1(obj, &js_sys::Array::from_iter(args.iter().cloned()))
        .map_err(JsErr::from)
}

async fn await_promise(p: Promise) -> Result<JsValue, JsErr> {
    JsFuture::from(p).await.map_err(JsErr::from)
}

fn slice_to_u8_array(slice: &[u8]) -> Uint8Array {
    Uint8Array::from(slice)
}

fn js_to_vec_u8(v: &JsValue) -> Result<Vec<u8>, JsErr> {
    if v.is_instance_of::<Uint8Array>() {
        Ok(Uint8Array::from(v.to_owned()).to_vec())
    } else if v.is_instance_of::<js_sys::ArrayBuffer>() {
        Ok(Uint8Array::new(v).to_vec())
    } else {
        Err(JsErr("Expected Uint8Array or ArrayBuffer".into()))
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct JsRequestSigner {
    obj: JsValue,
}

#[wasm_bindgen]
impl JsRequestSigner {
    /// Accept any JS object that implements the RequestSigner interface (see TS below).
    #[wasm_bindgen(constructor)]
    pub fn new(obj: JsValue) -> JsRequestSigner {
        JsRequestSigner { obj }
    }
}

impl std::fmt::Debug for JsRequestSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsRequestSigner").finish()
    }
}

#[async_trait(?Send)]
impl RequestSigner for JsRequestSigner {
    type Error = JsErr;

    fn alg(&self) -> Result<String, Self::Error> {
        // Allow either a string return or a Promise<string>.
        let out = call_method(&self.obj, "alg", &[])?;
        if out.is_instance_of::<Promise>() {
            // In a sync method, we can’t await; prefer requiring sync for `alg()`.
            // If you *really* need async here, change the trait to async and await.
            return Err(JsErr(
                "`alg()` must be synchronous or call try_sign/async variant".into(),
            ));
        }
        out.as_string()
            .ok_or_else(|| JsErr("`alg()` must return a string".into()))
    }

    fn jwk(&self) -> Result<JWK, Self::Error> {
        let out = call_method(&self.obj, "jwk", &[])?;
        if out.is_instance_of::<Promise>() {
            return Err(JsErr(
                "`jwk()` must be synchronous; make it return a plain object".into(),
            ));
        }
        serde_wasm_bindgen::from_value::<JWK>(out).map_err(|e| JsErr(e.to_string()))
    }

    async fn sign(&self, payload: &[u8]) -> Vec<u8> {
        let arg = slice_to_u8_array(payload);
        let out = match call_method(&self.obj, "sign", &[arg.into()]) {
            Ok(v) => v,
            Err(_e) => return vec![], // or panic/log; you can also bubble via a different signature
        };
        let resolved = if out.is_instance_of::<Promise>() {
            match await_promise(Promise::from(out)).await {
                Ok(v) => v,
                Err(_) => return vec![],
            }
        } else {
            out
        };
        js_to_vec_u8(&resolved).unwrap_or_default()
    }

    async fn try_sign(&self, payload: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // Prefer a dedicated try_sign if provided. Otherwise fall back to sign().
        let has_try =
            Reflect::has(&self.obj, &JsValue::from_str("try_sign")).map_err(JsErr::from)?;
        if has_try {
            let arg = slice_to_u8_array(payload);
            let out = call_method(&self.obj, "try_sign", &[arg.into()])?;
            let resolved = if out.is_instance_of::<Promise>() {
                await_promise(Promise::from(out)).await?
            } else {
                out
            };
            js_to_vec_u8(&resolved)
        } else {
            Ok(self.sign(payload).await)
        }
    }
}

#[wasm_bindgen]
pub fn make_request_signer(obj: JsValue) -> JsRequestSigner {
    JsRequestSigner::new(obj)
}
