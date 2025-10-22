use super::client::{OID4VPClient, OID4VPVerifier};
use crate::types::DCAPINamespaceRequest;

use std::collections::HashMap;

use anyhow::{Context, Result};
use base64::prelude::*;
use ciborium::Value as Cbor;
use http::StatusCode;
use isomdl::{
    cbor,
    definitions::{
        DeviceResponse, helpers::ByteStr, session::SessionTranscript,
        x509::trust_anchor::TrustAnchorRegistry,
    },
    presentation::{
        authentication::ResponseAuthenticationOutcome, reader::parse,
        reader_utils::validate_response,
    },
};
use openid4vp::{
    core::{
        authorization_request::parameters::{
            ClientMetadata, ExpectedOrigins, Nonce, ResponseMode, ResponseType,
        },
        credential_format::{ClaimFormatDesignation, ClaimFormatMap, ClaimFormatPayload},
        dcql_query::{
            DcqlCredentialClaimsQuery, DcqlCredentialClaimsQueryPath, DcqlCredentialQuery,
            DcqlCredentialSetQuery, DcqlQuery,
        },
        metadata::parameters::verifier::VpFormats,
        object::UntypedObject,
    },
    utils::NonEmptyVec,
    verifier::client::Client,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use sha2::{Digest, Sha256};
use tracing::debug;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InitiatedSessionState {
    oid4vp_session_id: String,
    origin: String,
    nonce: String,
}

#[derive(Serialize)]
pub struct InitiateResponse {
    pub request_jwt: String,
}

pub async fn initiate_inner(
    verifier: &OID4VPVerifier,
    request: &DCAPINamespaceRequest,
    session_id: String,
) -> Result<(InitiateResponse, InitiatedSessionState)> {
    let mut dcql_credential_query =
        DcqlCredentialQuery::new("0".into(), ClaimFormatDesignation::MsoMDoc);
    dcql_credential_query.set_meta(Some(
        [(
            "doctype_value".to_string(),
            serde_json::Value::String("org.iso.18013.5.1.mDL".to_string()),
        )]
        .into_iter()
        .collect(),
    ));
    let mut claims = Vec::new();
    for (doc_type, namespaces) in request.namespaces.clone().into_inner() {
        for namespace in namespaces.into_inner() {
            let mut dcql_credential_claims_query = DcqlCredentialClaimsQuery::new(
                vec![
                    DcqlCredentialClaimsQueryPath::String(doc_type.clone()),
                    DcqlCredentialClaimsQueryPath::String(namespace.clone()),
                ]
                .try_into()
                .unwrap(),
            );
            dcql_credential_claims_query.set_intent_to_retain(Some(true)); // Hardcoded to true to assume we don't know what the RP will do with it.
            claims.push(dcql_credential_claims_query);
        }
    }
    dcql_credential_query.set_claims(Some(claims.try_into().unwrap()));
    let mut dcql_credential_set_query =
        DcqlCredentialSetQuery::new(NonEmptyVec::new(vec!["0".into()]));
    dcql_credential_set_query.set_required(None);
    dcql_credential_set_query.set_purpose(Some(serde_json::Value::String(
        "Authorize to the government using your mobile drivers license".into(),
    )));
    let mut dcql_query = DcqlQuery::new(NonEmptyVec::new(dcql_credential_query));
    dcql_query.set_credential_sets(Some(NonEmptyVec::new(dcql_credential_set_query)));
    // Set the `vp_formats` parameter in the client metadata.
    let mut vp_formats = ClaimFormatMap::new();
    vp_formats.insert(
        ClaimFormatDesignation::MsoMDoc,
        ClaimFormatPayload::Alg(vec!["ES256".into()]),
    );
    let mut client_metadata = UntypedObject::default();
    client_metadata.insert(VpFormats(vp_formats));

    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).expect("Failed to generate nonce");
    let nonce = BASE64_URL_SAFE_NO_PAD.encode(nonce);

    let uuid: Uuid = session_id.parse().context("session id must be Uuid type")?;
    let (oid4vp_session_id, request_jwt) = verifier
        .0
        .build_authorization_request()
        .with_dcql_query(dcql_query)
        .with_request_parameter(ClientMetadata(client_metadata))
        .with_request_parameter(ResponseType::VpToken)
        .with_request_parameter(Nonce::from(nonce.clone()))
        .with_request_parameter(ResponseMode::DcApi)
        .with_request_parameter(ExpectedOrigins(vec![request.origin.clone()]))
        .build_dc_api_with_session_id(uuid)
        .await
        .context("Failed to build authorization request")?;
    debug!(
        "request_jwt: {}",
        serde_json::to_string(&request_jwt.clone()).unwrap()
    );

    let res = InitiateResponse { request_jwt };
    let state = InitiatedSessionState {
        oid4vp_session_id: oid4vp_session_id.to_string(),
        origin: request.origin.clone(),
        nonce,
    };

    Ok((res, state))
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DCAPIResponseData {
    vp_token: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTranscriptDCAPI<H>(Cbor, Cbor, H);

impl<H: Serialize + DeserializeOwned> SessionTranscript for SessionTranscriptDCAPI<H> {}

impl<H> SessionTranscriptDCAPI<H> {
    fn new(handover: H) -> Self {
        Self(Cbor::Null, Cbor::Null, handover)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handover(String, ByteStr);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoverInfo(String, String, String);

impl Handover {
    pub fn new(origin: String, client_id: String, nonce: String) -> anyhow::Result<Self> {
        let handover_info = HandoverInfo(origin, client_id, nonce);
        let handover_info_bytes = cbor::to_vec(&handover_info)?;
        let handover_info_hash = ByteStr::from(Sha256::digest(handover_info_bytes).to_vec());
        Ok(Handover(
            "OpenID4VPDCAPIHandover".to_string(),
            handover_info_hash,
        ))
    }
}

pub async fn submit_dc_response_inner(
    state: InitiatedSessionState,
    client: OID4VPClient,
    trust_anchor_registry: TrustAnchorRegistry,
    dc_response: DCAPIResponseData,
) -> Result<ResponseAuthenticationOutcome, (StatusCode, serde_json::Value)> {
    let first_vp_token = dc_response.vp_token.values().next().unwrap();
    let decoded_vp_token = BASE64_URL_SAFE_NO_PAD.decode(first_vp_token).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            json!({"error": format!("{:?}", e)}),
        )
    })?;
    let device_response: DeviceResponse = cbor::from_slice(&decoded_vp_token).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            json!({"error": format!("{:?}", e)}),
        )
    })?;
    let mut validated_response = ResponseAuthenticationOutcome::default();
    let handover = Handover::new(state.origin.clone(), client.id().0.clone(), state.nonce)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                json!({"error": format!("failed to create a handover: {e:?}")}),
            )
        })?;
    let session_transcript = SessionTranscriptDCAPI::new(handover);
    let validation_results = match parse(&device_response) {
        Ok((document, x5chain, namespaces)) => validate_response(
            session_transcript,
            trust_anchor_registry,
            x5chain,
            document.clone(),
            namespaces,
        ),
        Err(e) => {
            validated_response
                .errors
                .insert("parsing_errors".to_string(), json!(vec![format!("{e:?}")]));
            validated_response
        }
    };
    Ok(validation_results)
}
