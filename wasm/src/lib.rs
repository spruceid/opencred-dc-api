use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn initiate_dc_api(
    // config: JsValue,
    // request: JsValue,
    // session: JsValue,
    user_agent: String,
) -> Result<String, JsValue> {

    // TOOD: Add logging support.

    Ok(user_agent)

}
