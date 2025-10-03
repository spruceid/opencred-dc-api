use std::fmt::Debug;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use dc_api_core::openid4vp::verifier::session::{Session, SessionStore, Status};
use dc_api_core::openid4vp_frontend::Outcome;
use serde_wasm_bindgen;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Oid4VpSessionStore")]
    pub type Oid4VpSessionStore;

    #[wasm_bindgen(method, catch)]
    async fn initiate(this: &Oid4VpSessionStore, session: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    async fn updateStatus(
        this: &Oid4VpSessionStore,
        uuid: String,
        status: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    async fn getSession(this: &Oid4VpSessionStore, uuid: String) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    async fn removeSession(this: &Oid4VpSessionStore, uuid: String) -> Result<JsValue, JsValue>;
}

/// WebAssembly-compatible session store that delegates to JavaScript storage implementations.
///
/// This allows the session store to use any JavaScript storage backend (localStorage,
/// IndexedDB, external databases, etc.) by implementing the required methods in JavaScript.
#[wasm_bindgen]
#[derive(Clone)]
pub struct JsOid4VpSessionStore {
    store: Arc<Oid4VpSessionStore>,
}

#[wasm_bindgen]
impl JsOid4VpSessionStore {
    /// Creates a new WebAssembly session store with JavaScript storage implementation.
    ///
    /// # Parameters
    ///
    /// * `store` - JavaScript object implementing the Oid4VpSessionStore interface
    ///
    /// # Example JavaScript Usage
    ///
    /// ```javascript
    /// class MySessionStore {
    ///   async initiate(session) {
    ///     // Store session in your preferred storage
    ///     localStorage.setItem(`session_${session.uuid}`, JSON.stringify(session));
    ///   }
    ///
    ///   async updateStatus(uuid, status) {
    ///     // Update session status
    ///     const session = JSON.parse(localStorage.getItem(`session_${uuid}`));
    ///     session.status = status;
    ///     localStorage.setItem(`session_${uuid}`, JSON.stringify(session));
    ///   }
    ///
    ///   async getSession(uuid) {
    ///     // Get session from storage
    ///     const sessionData = localStorage.getItem(`session_${uuid}`);
    ///     if (!sessionData) throw new Error('Session not found');
    ///     return JSON.parse(sessionData);
    ///   }
    ///
    ///   async removeSession(uuid) {
    ///     // Remove session from storage
    ///     localStorage.removeItem(`session_${uuid}`);
    ///   }
    /// }
    ///
    /// const sessionStore = new WasmOid4VpSession(new MySessionStore());
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(store: Oid4VpSessionStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// Helper function to create a simple in-memory session store for testing purposes.
    ///
    /// This creates a JavaScript Map-based session store that can be used for development
    /// and testing without requiring external storage setup.
    ///
    /// # Example JavaScript Usage
    ///
    /// ```javascript
    /// import { WasmOid4VpSession } from './pkg/dc_api_wasm.js';
    ///
    /// const sessionStore = WasmOid4VpSession.createMemoryStore();
    /// // Use sessionStore with your DcApi instance
    /// ```
    #[wasm_bindgen(js_name = createMemoryStore)]
    pub fn create_memory_store() -> Result<JsOid4VpSessionStore, JsValue> {
        let js_code = r#"
            (() => {
                // Create a Map to store sessions in memory
                const sessionMap = new Map();

                return {
                    async initiate(session) {
                        sessionMap.set(session.uuid, session);
                    },

                    async updateStatus(uuid, status) {
                        const session = sessionMap.get(uuid);
                        if (!session) {
                            throw new Error(`Session not found: ${uuid}`);
                        }
                        session.status = status;
                        sessionMap.set(uuid, session);
                    },

                    async getSession(uuid) {
                        const session = sessionMap.get(uuid);
                        if (!session) {
                            throw new Error(`Session not found: ${uuid}`);
                        }
                        return session;
                    },

                    async removeSession(uuid) {
                        if (!sessionMap.delete(uuid)) {
                            throw new Error(`Session not found: ${uuid}`);
                        }
                    }
                };
            })()
        "#;

        let store_obj = js_sys::eval(js_code)
            .map_err(|e| JsValue::from_str(&format!("Failed to create memory store: {:?}", e)))?;

        let store = Oid4VpSessionStore::from(store_obj);
        Ok(JsOid4VpSessionStore::new(store))
    }

    /// Utility functions for session management from JavaScript
    /// Create a new UUID for session identification
    #[wasm_bindgen(js_name = generateSessionUuid)]
    pub fn generate_session_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    /// Parse a UUID string and validate it
    #[wasm_bindgen(js_name = parseUuid)]
    pub fn parse_uuid(uuid_str: &str) -> Result<String, JsValue> {
        match Uuid::parse_str(uuid_str) {
            Ok(uuid) => Ok(uuid.to_string()),
            Err(e) => Err(JsValue::from_str(&format!("Invalid UUID: {}", e))),
        }
    }

    /// Convert a Session to a JavaScript object
    #[wasm_bindgen(js_name = sessionToJs)]
    pub fn session_to_js(session: JsValue) -> Result<JsValue, JsValue> {
        // This function allows JavaScript to work with Session objects
        // The session parameter should be a serialized Session struct
        let session: Session = serde_wasm_bindgen::from_value(session)?;
        serde_wasm_bindgen::to_value(&session).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Convert a Status to a JavaScript object
    #[wasm_bindgen(js_name = statusToJs)]
    pub fn status_to_js(status: JsValue) -> Result<JsValue, JsValue> {
        // This function allows JavaScript to work with Status objects
        let status: Status = serde_wasm_bindgen::from_value(status)?;
        serde_wasm_bindgen::to_value(&status).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Helper function to create a Status::SentRequestByReference
    #[wasm_bindgen(js_name = createStatusSentRequestByReference)]
    pub fn create_status_sent_request_by_reference() -> JsValue {
        serde_wasm_bindgen::to_value(&Status::SentRequestByReference).unwrap()
    }

    /// Helper function to create a Status::SentRequest
    #[wasm_bindgen(js_name = createStatusSentRequest)]
    pub fn create_status_sent_request() -> JsValue {
        serde_wasm_bindgen::to_value(&Status::SentRequest).unwrap()
    }

    /// Helper function to create a Status::ReceivedResponse
    #[wasm_bindgen(js_name = createStatusReceivedResponse)]
    pub fn create_status_received_response() -> JsValue {
        serde_wasm_bindgen::to_value(&Status::ReceivedResponse).unwrap()
    }

    /// Helper function to create a Status::Complete with success outcome
    #[wasm_bindgen(js_name = createStatusCompleteSuccess)]
    pub fn create_status_complete_success(info: JsValue) -> Result<JsValue, JsValue> {
        let info = serde_wasm_bindgen::from_value(info)?;
        let outcome = Outcome::Success { info };
        let status = Status::Complete(outcome);
        serde_wasm_bindgen::to_value(&status).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Helper function to create a Status::Complete with failure outcome
    #[wasm_bindgen(js_name = createStatusCompleteFailure)]
    pub fn create_status_complete_failure(reason: &str) -> JsValue {
        let outcome = Outcome::Failure {
            reason: reason.to_string(),
        };
        let status = Status::Complete(outcome);
        serde_wasm_bindgen::to_value(&status).unwrap()
    }

    /// Helper function to create a Status::Complete with error outcome
    #[wasm_bindgen(js_name = createStatusCompleteError)]
    pub fn create_status_complete_error(cause: &str) -> JsValue {
        let outcome = Outcome::Error {
            cause: cause.to_string(),
        };
        let status = Status::Complete(outcome);
        serde_wasm_bindgen::to_value(&status).unwrap()
    }
}

impl Debug for JsOid4VpSessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsOid4VpSessionStore")
            .field("store", &"[JavaScript Oid4VpSessionStore]")
            .finish()
    }
}

#[cfg_attr(target_arch="wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl SessionStore for JsOid4VpSessionStore {
    async fn initiate(&self, session: Session) -> Result<()> {
        let session_value = serde_wasm_bindgen::to_value(&session)
            .map_err(|e| anyhow!("Failed to serialize session: {}", e))?;

        self.store
            .initiate(session_value)
            .await
            .map_err(|e| anyhow!("JavaScript error in initiate: {:?}", e))?;

        Ok(())
    }

    async fn update_status(&self, uuid: Uuid, status: Status) -> Result<()> {
        let status_value = serde_wasm_bindgen::to_value(&status)
            .map_err(|e| anyhow!("Failed to serialize status: {}", e))?;

        self.store
            .updateStatus(uuid.to_string(), status_value)
            .await
            .map_err(|e| anyhow!("JavaScript error in updateStatus: {:?}", e))?;

        Ok(())
    }

    async fn get_session(&self, uuid: Uuid) -> Result<Session> {
        let result = self
            .store
            .getSession(uuid.to_string())
            .await
            .map_err(|e| anyhow!("JavaScript error in getSession: {:?}", e))?;

        let session: Session = serde_wasm_bindgen::from_value(result)
            .map_err(|e| anyhow!("Failed to deserialize session: {}", e))?;

        Ok(session)
    }

    async fn remove_session(&self, uuid: Uuid) -> Result<()> {
        self.store
            .removeSession(uuid.to_string())
            .await
            .map_err(|e| anyhow!("JavaScript error in removeSession: {:?}", e))?;

        Ok(())
    }
}
