use std::collections::BTreeMap;

use anyhow::{Context, Result};
use base64::prelude::*;
use ciborium::Value as Cbor;
use coset::iana;
use hpke::{
    Deserializable, Kem, OpModeR, Serializable, aead::AesGcm128, kdf::HkdfSha256,
    kem::DhP256HkdfSha256,
};
use http::StatusCode;
use isomdl::{
    cbor,
    cose::sign1::PreparedCoseSign1,
    definitions::{
        CoseKey, DeviceResponse, DocRequest, EC2Curve, EC2Y,
        device_request::{DeviceRequest, DeviceRequestInfo, ItemsRequest, UseCase},
        helpers::{ByteStr, NonEmptyMap, NonEmptyVec, Tag24},
        session::SessionTranscript,
        x509::{X5Chain, trust_anchor::TrustAnchorRegistry, x5chain::X5CHAIN_COSE_HEADER_LABEL},
    },
    presentation::{
        authentication::ResponseAuthenticationOutcome,
        reader::{ReaderAuthenticationAll, parse, parse_namespaces},
        reader_utils::validate_response,
    },
};
use p256::ecdsa::signature::SignerMut;
use pkcs8::DecodePrivateKey;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    config::Oid4VpConfig,
    types::{DCAPINamespaceRequest, DCAPIRequestOrgIsoMDoc},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InitiatedSessionState {
    private_key: Vec<u8>,
    session_transcript_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EncryptionInfo(String, EncryptionParameters);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncryptionParameters {
    nonce: ByteStr,
    recipient_public_key: CoseKey,
}

pub async fn initiate_inner(
    config: &Oid4VpConfig,
    req: &DCAPINamespaceRequest,
) -> Result<(DCAPIRequestOrgIsoMDoc, InitiatedSessionState)> {
    // Hardcoded to true to assume we don't know what the RP will do with it.
    // unwraps are safe as the original data is non-empty
    let namespaces: BTreeMap<_, _> = req
        .namespaces
        .iter()
        .map(|(n, m)| {
            let m = m
                .iter()
                .map(|m| (m.clone(), true))
                .collect::<BTreeMap<String, bool>>();
            let m: NonEmptyMap<String, bool> = m.try_into().unwrap();
            (n.clone(), m)
        })
        .collect();
    let namespaces = namespaces.try_into().unwrap();
    let items_request = ItemsRequest {
        doc_type: "org.iso.18013.5.1.mDL".into(),
        namespaces,
        request_info: None,
    };
    let doc_request = DocRequest {
        reader_auth: None,
        items_request: Tag24::new(items_request.clone())
            .context("Could not build Tag24 items request")?,
    };
    let device_request_info = DeviceRequestInfo {
        use_cases: NonEmptyVec::new(UseCase {
            mandatory: true,
            document_sets: NonEmptyVec::new(NonEmptyVec::new(0)),
            purpose_hints: None,
        }),
    };

    let (private_key, public_key) = DhP256HkdfSha256::gen_keypair(&mut crate::rng::rng());
    let public_key_bytes = public_key.to_bytes();
    let (x, y) = public_key_bytes.as_slice()[1..].split_at(32);

    let cose_key = CoseKey::EC2 {
        crv: EC2Curve::P256,
        x: x.to_vec(),
        y: EC2Y::Value(y.to_vec()),
    };

    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).expect("Failed to generate nonce");

    let encryption_parameters = EncryptionParameters {
        nonce: nonce.to_vec().into(),
        recipient_public_key: cose_key,
    };
    let encryption_info = EncryptionInfo("dcapi".into(), encryption_parameters);
    let encryption_info_bytes =
        cbor::to_vec(&encryption_info).context("Could not serialize to cbor encryption info")?;
    let encryption_info_base64 = BASE64_URL_SAFE_NO_PAD.encode(encryption_info_bytes);

    let handover = Handover::new(encryption_info_base64.clone(), req.origin.clone())
        .context("failed to create a handover")?;
    let session_transcript = SessionTranscriptDCAPI::new(handover);
    let session_transcript_bytes = cbor::to_vec(&session_transcript)
        .context("Could not serialize to cbor session transcript")?;

    let reader_authentication_all = ReaderAuthenticationAll(
        "ReaderAuthenticationAll".into(),
        session_transcript,
        vec![Tag24::new(items_request).context("Failed to build tag 24 items request")?],
        Some(
            Tag24::new(device_request_info.clone())
                .context("Failed to build tag 25 device request info")?,
        ),
    );
    let reader_authentication_all_bytes = cbor::to_vec(
        &Tag24::new(reader_authentication_all)
            .context("Could not build tag 24 reader authentication all")?,
    )
    .context("Failed to serialize reader authentication all")?;

    let secret_key =
        p256::SecretKey::from_pkcs8_pem(&config.client.key).context("Could not load JWK")?;
    let mut signer: p256::ecdsa::SigningKey = secret_key.into();
    let protected = coset::HeaderBuilder::new()
        .algorithm(iana::Algorithm::ES256)
        .build();
    let mut x5chain_builder = X5Chain::builder();
    for cert in config.client.x5c.iter() {
        x5chain_builder = x5chain_builder
            .with_certificate(cert.clone())
            .context("Failed to add cert to chain")?;
    }
    let x5chain = x5chain_builder.build().context("Failed to build x5chain")?;
    let unprotected = coset::HeaderBuilder::new()
        .value(X5CHAIN_COSE_HEADER_LABEL, x5chain.into_cbor())
        .build();
    let builder = coset::CoseSign1Builder::new()
        .protected(protected)
        .unprotected(unprotected);
    let prepared =
        PreparedCoseSign1::new(builder, Some(&reader_authentication_all_bytes), None, false)
            .unwrap();
    let signature_payload = prepared.signature_payload();
    let signature: p256::ecdsa::Signature = signer.sign(signature_payload);
    let cose_sign1 = prepared.finalize(signature.to_bytes().to_vec());

    let reader_auth_all = NonEmptyVec::new(cose_sign1);

    let device_request = DeviceRequest {
        version: "1.1".to_string(),
        doc_requests: NonEmptyVec::new(doc_request),
        reader_auth_all: Some(reader_auth_all),
        device_request_info: Some(
            Tag24::new(device_request_info).context("Could not build tag25 device request info")?,
        ),
    };
    let device_request_bytes =
        cbor::to_vec(&device_request).context("Could not serialize to cbor device request")?;
    let device_request_base64 = BASE64_URL_SAFE_NO_PAD.encode(device_request_bytes);

    let res = DCAPIRequestOrgIsoMDoc {
        device_request: device_request_base64,
        encryption_info: encryption_info_base64,
    };
    let session_state = InitiatedSessionState {
        private_key: private_key.to_bytes().to_vec(),
        session_transcript_bytes,
    };

    Ok((res, session_state))
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DCAPIResponseData {
    response: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EncryptedResponse(String, EncryptedResponseData);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncryptedResponseData {
    enc: ByteStr,
    cipher_text: ByteStr,
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
pub struct HandoverInfo(String, String);

impl Handover {
    pub fn new(encryption_info_base64: String, origin: String) -> anyhow::Result<Self> {
        let handover_info = HandoverInfo(encryption_info_base64, origin);
        let handover_info_bytes = cbor::to_vec(&handover_info)?;
        let handover_info_hash = ByteStr::from(Sha256::digest(handover_info_bytes).to_vec());
        Ok(Handover("dcapi".to_string(), handover_info_hash))
    }
}

pub async fn submit_dc_response_inner(
    state: InitiatedSessionState,
    trust_anchor_registry: TrustAnchorRegistry,
    dc_response: DCAPIResponseData,
) -> Result<ResponseAuthenticationOutcome, (StatusCode, serde_json::Value)> {
    let response = dc_response.response;
    let response_bytes = BASE64_URL_SAFE_NO_PAD.decode(response).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            json!({"error": format!("{:?}", e)}),
        )
    })?;
    let encrypted_response: EncryptedResponse = cbor::from_slice(&response_bytes).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            json!({"error": format!("{:?}", e)}),
        )
    })?;

    let encapped_key = <hpke::kem::DhP256HkdfSha256 as hpke::Kem>::EncappedKey::from_bytes(
        encrypted_response.1.enc.as_ref(),
    )
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            json!({"error": format!("Could not deserialize encapped key: {:?}", e)}),
        )
    })?;

    let private_key =
        <hpke::kem::DhP256HkdfSha256 as hpke::Kem>::PrivateKey::from_bytes(&state.private_key)
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    json!({"error": format!("Could not deserialize private key: {e:?}")}),
                )
            })?;
    let mut decryption_context = hpke::setup_receiver::<AesGcm128, HkdfSha256, DhP256HkdfSha256>(
        &OpModeR::Base,
        &private_key,
        &encapped_key,
        &state.session_transcript_bytes,
    )
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            json!({"error": format!("Could not set up HPKE received: {e:?}")}),
        )
    })?;
    let device_response_bytes = decryption_context
        .open(encrypted_response.1.cipher_text.as_ref(), b"")
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                json!({"error": format!("Could not decrypt response: {e:?}")}),
            )
        })?;

    let device_response: DeviceResponse =
        cbor::from_slice(&device_response_bytes).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                json!({"error": format!("{:?}", e)}),
            )
        })?;
    let session_transcript: SessionTranscriptDCAPI<Handover> =
        cbor::from_slice(&state.session_transcript_bytes).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": format!("Could not deserialize stored session transcript: {e:?}")
                }),
            )
        })?;

    let mut validated_response = ResponseAuthenticationOutcome::default();
    let validation_results = match parse(&device_response) {
        Ok((document, x5chain, namespaces)) => validate_response(
            session_transcript,
            trust_anchor_registry,
            x5chain,
            document.clone(),
            namespaces,
        ),
        Err(e) => {
            if let Ok(namespaces) = parse_namespaces(&device_response) {
                validated_response.response = namespaces;
            }
            validated_response
                .errors
                .insert("parsing_errors".to_string(), json!(vec![format!("{e:?}")]));
            validated_response
        }
    };

    Ok(validation_results)
}
