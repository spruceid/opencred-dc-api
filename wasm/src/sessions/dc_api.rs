use anyhow::{Result, anyhow};
use async_trait::async_trait;
use dc_api_core::session::{DcApiSessionEngine, Session};
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "DcApiSessionStore")]
    pub type JsDcApiSessionStore;

    #[wasm_bindgen(method, catch)]
    async fn newSession(
        this: &JsDcApiSessionStore,
        session_id: String,
        session: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    async fn getSession(this: &JsDcApiSessionStore, id: String) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    async fn getSessionUnauthenticated(
        this: &JsDcApiSessionStore,
        id: String,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    async fn updateSession(
        this: &JsDcApiSessionStore,
        session_id: String,
        session: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    async fn removeSession(
        this: &JsDcApiSessionStore,
        session_id: String,
    ) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
pub struct JsDcApiSessionDriver {
    storage: JsDcApiSessionStore,
}

impl JsDcApiSessionDriver {
    pub fn new(storage: JsDcApiSessionStore) -> Self {
        Self { storage }
    }
}

#[cfg_attr(target_arch="wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl DcApiSessionEngine for JsDcApiSessionDriver {
    async fn new_session(&self, session_id: String, session: Session) -> Result<()> {
        let session_js = serde_wasm_bindgen::to_value(&session)
            .map_err(|e| anyhow!("Failed to serialize session: {}", e))?;

        self.storage
            .newSession(session_id, session_js)
            .await
            .map_err(|e| anyhow!("JavaScript error in newSession: {:?}", e))?;

        Ok(())
    }

    async fn get_session(&self, id: String, client_secret: String) -> Result<Option<Session>> {
        let result = self
            .storage
            .getSession(id)
            .await
            .map_err(|e| anyhow!("JavaScript error in getSession: {:?}", e))?;

        if result.is_null() || result.is_undefined() {
            return Ok(None);
        }

        let session: Session = serde_wasm_bindgen::from_value(result)
            .map_err(|e| anyhow!("Failed to deserialize Session: {}", e))?;

        Session::check_client_secret(&client_secret, &session.client_secret_hash)?;

        Ok(Some(session))
    }

    async fn get_session_unauthenticated(&self, id: String) -> Result<Option<Session>> {
        let result = self
            .storage
            .getSessionUnauthenticated(id)
            .await
            .map_err(|e| anyhow!("JavaScript error in getSessionUnauthenticated: {:?}", e))?;

        if result.is_null() || result.is_undefined() {
            return Ok(None);
        }

        let session: Session = serde_wasm_bindgen::from_value(result)
            .map_err(|e| anyhow!("Failed to deserialize Session: {}", e))?;

        Ok(Some(session))
    }

    async fn update_session(&self, session_id: String, session: Session) -> Result<()> {
        let session_js = serde_wasm_bindgen::to_value(&session)
            .map_err(|e| anyhow!("Failed to serialize session: {}", e))?;

        self.storage
            .updateSession(session_id, session_js)
            .await
            .map_err(|e| anyhow!("JavaScript error in updateSession: {:?}", e))?;

        Ok(())
    }

    async fn remove_session(&self, session_id: String) -> Result<()> {
        self.storage
            .removeSession(session_id)
            .await
            .map_err(|e| anyhow!("JavaScript error in removeSession: {:?}", e))?;

        Ok(())
    }
}
