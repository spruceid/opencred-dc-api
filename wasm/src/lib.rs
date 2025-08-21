use openid4vp::dc_api::DCAPIRequest;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn initiate_dc_api(
    // config: JsValue,
    request: JsValue,
    // session: JsValue,
    user_agent: String,
) -> Result<JsValue, JsValue> {
    let request: DCAPIRequest = serde_wasm_bindgen::from_value(request)?;

    // echo the request back

    let value = serde_wasm_bindgen::to_value(&request)?;

    Ok(value)
}
