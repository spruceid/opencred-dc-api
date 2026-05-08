#![cfg(target_arch = "wasm32")]
#![allow(clippy::arc_with_non_send_sync)]

pub mod sessions;

use std::sync::Arc;

use dc_api_core::client::{OID4VPClient, OID4VPVerifier};
use dc_api_core::config::{Client, Oid4VpConfig};
use dc_api_core::isomdl::definitions::x509::trust_anchor::{
    TrustAnchor, TrustAnchorRegistry, TrustPurpose,
};
use dc_api_core::session::{SessionCreationResponse, SessionState, SessionStorage};
use dc_api_core::types::{
    DCAPINamespaceRequest, DCAPIRequest, DCAPIRequestOpenId4VP, DCAPIRequests, DCAPIResponse,
};
use dc_api_core::url::Url;
use dc_api_core::x509_cert::certificate::CertificateInner;
use dc_api_core::{annex_c as ac, annex_d as ad};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

pub use sessions::JsOid4VpSessionStore;

use crate::sessions::{JsDcApiSessionDriver, JsDcApiSessionStore};

/// Configuration object for `DcApi.new`.
///
/// The `issuer` and `reader` certificate chains serve different trust roles
/// and must be supplied separately. The issuer chain populates the trust
/// anchor registry used to verify presented mDocs; the reader chain is
/// presented by the verifier when signing requests to the wallet.
#[derive(Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DcApiConfig {
    /// PKCS#8 PEM-encoded private key used to sign requests to the wallet.
    pub key: String,
    /// Base URL of the verifier (e.g. `https://verifier.example.com`).
    pub base_url: String,
    /// Endpoint that wallets POST presentation responses to.
    pub submission_endpoint: String,
    /// Endpoint serving request-by-reference (`request_uri`) JWTs.
    pub reference_endpoint: String,
    /// PEM-encoded chain of trusted issuer CAs. Used as the trust anchor
    /// registry when verifying presented mDocs.
    #[tsify(type = "Uint8Array")]
    #[serde(with = "serde_bytes")]
    pub issuer_ca_x5c_pem: Vec<u8>,
    /// PEM-encoded chain for the reader/verifier client certificate. Sent to
    /// the wallet as part of request signing.
    #[tsify(type = "Uint8Array")]
    #[serde(with = "serde_bytes")]
    pub reader_ca_x5c_pem: Vec<u8>,
}

#[wasm_bindgen(typescript_custom_section)]
const TS_RESPONSE_AUTHENTICATION_OUTCOME: &'static str = r#"
export type AuthenticationStatus = "Unchecked" | "Invalid" | "Valid";

export type Errors = Record<string, Record<string, unknown>>;

export interface ResponseAuthenticationOutcome {
  response: Record<string, Record<string, unknown>>;
  doc_types: string[];
  issuer_authentication: AuthenticationStatus;
  device_authentication: AuthenticationStatus;
  errors: Errors;
  warnings: Errors;
}
"#;

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
        config: DcApiConfig,
        oid4vp_session_store: JsOid4VpSessionStore,
        js_dc_api_session_store: JsDcApiSessionStore,
    ) -> Result<DcApi, JsValue> {
        let DcApiConfig {
            key,
            base_url,
            submission_endpoint,
            reference_endpoint,
            issuer_ca_x5c_pem,
            reader_ca_x5c_pem,
        } = config;

        let issuer_ca_x5c = CertificateInner::load_pem_chain(&issuer_ca_x5c_pem)
            .map_err(|e| JsValue::from(e.to_string()))?;

        let reader_ca_x5c = CertificateInner::load_pem_chain(&reader_ca_x5c_pem)
            .map_err(|e| JsValue::from(e.to_string()))?;

        let base_url = base_url
            .parse::<Url>()
            .map_err(|e| JsValue::from(e.to_string()))?;
        let oid4vp_config = Oid4VpConfig {
            base_url,
            submission_endpoint,
            reference_endpoint,
            client: Client {
                key,
                x5c: reader_ca_x5c,
            },
        };
        let oid4vp_client =
            OID4VPClient::new(&oid4vp_config).map_err(|e| JsValue::from(e.to_string()))?;
        let verifier = OID4VPVerifier::new(
            &oid4vp_config,
            oid4vp_client,
            Arc::new(oid4vp_session_store),
        )
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

        let dc_api_session =
            SessionStorage::new(Arc::new(JsDcApiSessionDriver::new(js_dc_api_session_store)));

        let trust_anchor_registry = TrustAnchorRegistry {
            anchors: issuer_ca_x5c
                .into_iter()
                .map(|certificate| TrustAnchor {
                    certificate,
                    purpose: TrustPurpose::ReaderCa,
                })
                .collect(),
        };

        Ok(Self {
            verifier,
            config: oid4vp_config,
            dc_api_session,
            trust_anchor_registry,
        })
    }

    #[wasm_bindgen(js_name = createNewSession)]
    pub async fn create_new_session(&self) -> Result<SessionCreationResponse, JsValue> {
        let creation = self
            .dc_api_session
            .new_session()
            .await
            .map_err(|e| JsValue::from(format!("Failed to save new dc-api session: {e:?}")))?;

        Ok(creation.session_creation_response)
    }

    #[wasm_bindgen(js_name = initiateRequest)]
    pub async fn initiate_request(
        &self,
        session_id: String,
        session_secret: String,
        request: DCAPINamespaceRequest,
        user_agent: Option<String>,
    ) -> Result<DCAPIRequests, JsValue> {
        let mut session = self
            .dc_api_session
            .get_session(session_id.clone(), session_secret)
            .await
            .map_err(|e| JsValue::from(e.to_string()))?
            .ok_or(JsValue::from_str("Session not found"))?;

        if session.state != SessionState::Created {
            return Err(JsValue::from_str("Invalid session"));
        }

        let (annexc_res, annexc_state) = ac::initiate_inner(&self.config, &request)
            .await
            .map_err(|e| JsValue::from(e.to_string()))?;
        let (annexd_res, annexd_state) =
            ad::initiate_inner(&self.verifier, &request, session_id.clone())
                .await
                .map_err(|e| JsValue::from(e.to_string()))?;

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

        Ok(DCAPIRequests { requests })
    }

    #[wasm_bindgen(js_name = submitResponse, unchecked_return_type = "ResponseAuthenticationOutcome")]
    pub async fn submit_response(
        &self,
        session_id: String,
        session_secret: String,
        response: DCAPIResponse,
    ) -> Result<JsValue, JsValue> {
        let mut session = self
            .dc_api_session
            .get_session(session_id.clone(), session_secret)
            .await
            .map_err(|e| JsValue::from(e.to_string()))?
            .ok_or(JsValue::from_str("Session not found"))?;

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

                Ok(serde_wasm_bindgen::to_value(&res)?)
            }
            _ => Err(JsValue::from_str("Invalid Session")),
        }
    }
}
