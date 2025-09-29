use crate::sessions::dc_api::{JsDcApiSessionEngine, JsSessionStorage};
use core::session::{SessionCreationResponse, SessionStorage as CoreSessionStorage};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SessionStorage {
    inner: CoreSessionStorage,
}

#[wasm_bindgen]
impl SessionStorage {
    #[wasm_bindgen(constructor)]
    pub fn new(js_storage: JsSessionStorage) -> Self {
        let engine = JsDcApiSessionEngine::new(js_storage);
        let inner = CoreSessionStorage::new(Arc::new(engine));
        Self { inner }
    }

    #[wasm_bindgen(js_name = "newSession")]
    pub async fn new_session(&self) -> Result<JsValue, JsError> {
        let response = self
            .inner
            .new_session()
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&response).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "getSession")]
    pub async fn get_session(&self, id: String, client_secret: String) -> Result<JsValue, JsError> {
        let session = self
            .inner
            .get_session(id, &client_secret)
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&session).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "getSessionUnauthenticated")]
    pub async fn get_session_unauthenticated(&self, id: String) -> Result<JsValue, JsError> {
        let session = self
            .inner
            .get_session_unauthenticated(id)
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&session).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "updateSession")]
    pub async fn update_session(
        &self,
        session_id: String,
        session_js: JsValue,
    ) -> Result<(), JsError> {
        let session =
            serde_wasm_bindgen::from_value(session_js).map_err(|e| JsError::new(&e.to_string()))?;

        self.inner
            .update_session(session_id, session)
            .await
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "removeSession")]
    pub async fn remove_session(&self, session_id: String) -> Result<(), JsError> {
        self.inner
            .remove_session(session_id)
            .await
            .map_err(|e| JsError::new(&e.to_string()))
    }
}
