#[cfg(target_arch = "wasm32")]
pub mod signer;

use dc_api_core::request::DCAPIRequest;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn initiate_dc_api(
    // config: JsValue,
    request: JsValue,
    // session: JsValue,
    _user_agent: String,
) -> Result<JsValue, JsValue> {
    let request: DCAPIRequest = serde_wasm_bindgen::from_value(request)?;
    let value = serde_wasm_bindgen::to_value(&request)?;

    Ok(value)
}
