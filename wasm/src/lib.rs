#![cfg(target_arch = "wasm32")]
#![allow(clippy::arc_with_non_send_sync)]

pub mod sessions;

use std::sync::Arc;

use dc_api_core::client::{OID4VPClient, OID4VPVerifier};
use dc_api_core::config::{Client, Oid4VpConfig};
use dc_api_core::isomdl::definitions::x509::trust_anchor::{
    TrustAnchor, TrustAnchorRegistry, TrustPurpose,
};
use dc_api_core::session::{SessionState, SessionStorage};
use dc_api_core::types::{
    DCAPINamespaceRequest, DCAPIRequest, DCAPIRequestOpenId4VP, DCAPIRequests, DCAPIResponse,
};
use dc_api_core::url::Url;
use dc_api_core::x509_cert::certificate::CertificateInner;
use dc_api_core::{annex_c as ac, annex_d as ad};
use wasm_bindgen::prelude::*;

pub use sessions::JsOid4VpSessionStore;

use crate::sessions::{JsDcApiSessionDriver, JsDcApiSessionStore};

#[wasm_bindgen]
pub struct DcApi {
    verifier: OID4VPVerifier,
    config: Oid4VpConfig,
    dc_api_session: SessionStorage,
    trust_anchor_registry: TrustAnchorRegistry,
}

#[wasm_bindgen]
impl DcApi {
    #[wasm_bindgen]
    pub async fn new(
        key: String,
        base_url: String,
        submission_endpoint: String,
        reference_endpoint: String,
        cert_chain_pem: Vec<u8>,
        oid4vp_session_store: JsOid4VpSessionStore,
        js_dc_api_session_store: JsDcApiSessionStore,
    ) -> Result<Self, JsValue> {
        let x5c = CertificateInner::load_pem_chain(&cert_chain_pem)
            .map_err(|e| JsValue::from(e.to_string()))?;

        let base_url = base_url
            .parse::<Url>()
            .map_err(|e| JsValue::from(e.to_string()))?;
        let config = Oid4VpConfig {
            base_url,
            submission_endpoint,
            reference_endpoint,
            client: Client {
                key,
                x5c: x5c.clone(),
            },
        };
        let oid4vp_client = OID4VPClient::new(&config).map_err(|e| JsValue::from(e.to_string()))?;
        let verifier = OID4VPVerifier::new(&config, oid4vp_client, Arc::new(oid4vp_session_store))
            .await
            .map_err(|e| JsValue::from(e.to_string()))?;

        let dc_api_session =
            SessionStorage::new(Arc::new(JsDcApiSessionDriver::new(js_dc_api_session_store)));

        // Construct trust anchor registry
        let trust_anchor_registry = TrustAnchorRegistry {
            anchors: x5c
                .into_iter()
                .map(|certificate| TrustAnchor {
                    certificate,
                    purpose: TrustPurpose::ReaderCa,
                })
                .collect(),
        };

        Ok(Self {
            verifier,
            config,
            dc_api_session,
            trust_anchor_registry,
        })
    }

    #[wasm_bindgen]
    pub async fn create_new_session(&self) -> Result<JsValue, JsValue> {
        let session = self
            .dc_api_session
            .new_session()
            .await
            .map_err(|e| JsValue::from(format!("Failed to save new dc-api session: {e:?}")))?;

        let value = serde_wasm_bindgen::to_value(&session)?;

        Ok(value)
    }

    #[wasm_bindgen]
    pub async fn initiate_request(
        &self,
        session_id: String,
        session_secret: String,
        request: JsValue,
        user_agent: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let mut session = self
            .dc_api_session
            .get_session(session_id.clone(), session_secret.clone())
            .await
            .map_err(|e| JsValue::from(e.to_string()))?
            .ok_or(JsValue::from_str("Session not found"))?;

        if session.state != SessionState::Created {
            return Err(JsValue::from_str("Invalid session"));
        }

        let request: DCAPINamespaceRequest = serde_wasm_bindgen::from_value(request)?;

        let (annexc_res, annexc_state) = ac::initiate_inner(&self.config, &request)
            .await
            .map_err(|e| JsValue::from(e.to_string()))?;
        let (annexd_res, annexd_state) = ad::initiate_inner(&self.verifier, &request)
            .await
            .map_err(|e| JsValue::from(e.to_string()))?;

        // Update the session state with the initiate annex states
        session.state = SessionState::Initiated {
            annex_c: annexc_state,
            annex_d: annexd_state,
        };

        self.dc_api_session
            .update_session(session_id, session)
            .await
            .map_err(|e| JsValue::from(e.to_string()))?;

        let mut requests = vec![DCAPIRequest::OpenId4VP {
            data: DCAPIRequestOpenId4VP {
                request: annexd_res.request_jwt,
            },
        }];

        // NOTE: Chrome crashes on processing the DC API response
        // when the request contains more than one request.
        if let Some(user_agent) = user_agent
            && !user_agent.as_str().contains("Chrome")
        {
            requests.push(DCAPIRequest::OrgIsoMDoc { data: annexc_res });
        }

        let value = serde_wasm_bindgen::to_value(&DCAPIRequests { requests })?;

        Ok(value)
    }

    #[wasm_bindgen]
    pub async fn submit_response(
        &self,
        session_id: String,
        session_secret: String,
        response: JsValue,
    ) -> Result<JsValue, JsValue> {
        let mut session = self
            .dc_api_session
            .get_session(session_id.clone(), session_secret.clone())
            .await
            .map_err(|e| JsValue::from(e.to_string()))?
            .ok_or(JsValue::from_str("Session not found"))?;

        let response: DCAPIResponse = serde_wasm_bindgen::from_value(response)?;

        match session.state {
            SessionState::Initiated { annex_c, annex_d } => {
                let res = match response {
                    DCAPIResponse::OrgIsoMDoc { data } => ac::submit_dc_response_inner(
                        annex_c,
                        self.trust_anchor_registry.clone(),
                        data,
                    )
                    .await
                    .map_err(|(status, e)| {
                        JsValue::from(format!("failed with status {status:?} and error: {e:?}"))
                    })?,
                    DCAPIResponse::OpenId4VP { data } => {
                        let client = OID4VPClient::new(&self.config)
                            .map_err(|e| JsValue::from(e.to_string()))?;
                        ad::submit_dc_response_inner(
                            annex_d,
                            client,
                            self.trust_anchor_registry.clone(),
                            data,
                        )
                        .await
                        .map_err(|(status, e)| {
                            JsValue::from(format!("failed with status {status:?} and error: {e:?}"))
                        })?
                    }
                };

                session.state = SessionState::Completed(res.clone());
                self.dc_api_session
                    .update_session(session_id, session)
                    .await
                    .map_err(|e| JsValue::from(e.to_string()))?;

                let response = serde_wasm_bindgen::to_value(&res)?;

                Ok(response)
            }
            _ => Err(JsValue::from_str("Invalid Session")),
        }
    }
}
