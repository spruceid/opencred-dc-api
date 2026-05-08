use std::sync::Arc;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use base64::prelude::*;
use openid4vp::{
    core::authorization_request::{
        AuthorizationRequestObject,
        parameters::{ClientId, ClientIdScheme},
    },
    verifier::{
        client::{Client, X509SanVariant},
        request_signer::RequestSigner,
    },
};
use pkcs8::der::Encode;
use serde_json::json;
use tracing::debug;
use x509_cert::{
    Certificate,
    ext::pkix::{SubjectAltName, name::GeneralName},
};

// TODO this is redefined to add the scheme prefix

/// A [Client] with the `x509_san_dns` or `x509_san_uri` Client Identifier.
#[derive(Debug, Clone)]
pub struct X509SanClient {
    id: ClientId,
    x5c: Vec<Certificate>,
    signer: Arc<dyn RequestSigner<Error = anyhow::Error> + Send + Sync>,
    variant: X509SanVariant,
}

impl X509SanClient {
    pub fn new(
        x5c: Vec<Certificate>,
        signer: Arc<dyn RequestSigner<Error = anyhow::Error> + Send + Sync>,
        variant: X509SanVariant,
    ) -> Result<Self> {
        let leaf = &x5c[0];
        let id = if let Some(san) = leaf
            .tbs_certificate
            .filter::<SubjectAltName>()
            .filter_map(|r| match r {
                Ok((_crit, san)) => Some(san.0.into_iter()),
                Err(e) => {
                    debug!("unable to parse SubjectAlternativeName from DER: {e}");
                    None
                }
            })
            .flatten()
            .filter_map(|general_name| match (general_name, variant) {
                (GeneralName::DnsName(uri), X509SanVariant::Dns) => Some(uri.to_string()),
                (gn, X509SanVariant::Dns) => {
                    debug!("found non-DNS SAN: {gn:?}");
                    None
                }
                (GeneralName::UniformResourceIdentifier(uri), X509SanVariant::Uri) => {
                    Some(uri.to_string())
                }
                (gn, X509SanVariant::Uri) => {
                    debug!("found non-URI SAN: {gn:?}");
                    None
                }
            })
            .next()
        {
            san
        } else {
            bail!("x509 certificate does not contain Subject Alternative Name");
        };
        let variant_str = match variant {
            X509SanVariant::Dns => "dns",
            X509SanVariant::Uri => "uri",
        };
        Ok(X509SanClient {
            id: ClientId(format!("x509_san_{variant_str}:{id}")),
            x5c,
            signer,
            variant,
        })
    }
}

#[cfg_attr(target_arch="wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Client for X509SanClient {
    fn id(&self) -> &ClientId {
        &self.id
    }

    fn scheme(&self) -> ClientIdScheme {
        self.variant.to_scheme()
    }

    async fn generate_request_object_jwt(
        &self,
        body: &AuthorizationRequestObject,
    ) -> Result<String> {
        let algorithm = self
            .signer
            .alg()
            .context("failed to retrieve signing algorithm")?;
        let x5c: Vec<String> = self
            .x5c
            .iter()
            .map(|x509| x509.to_der())
            .map(|der| Ok(BASE64_STANDARD.encode(der?)))
            .collect::<Result<_>>()?;
        let header = json!({
            "alg": algorithm,
            "x5c": x5c,
            "typ": "oauth-authz-req+jwt"
        });
        make_jwt(header, body, self.signer.as_ref()).await
    }
}

async fn make_jwt<S: RequestSigner + ?Sized>(
    header: serde_json::Value,
    body: &AuthorizationRequestObject,
    signer: &S,
) -> Result<String> {
    let header_b64: String =
        serde_json::to_vec(&header).map(|b| BASE64_URL_SAFE_NO_PAD.encode(b))?;
    let body_b64 = serde_json::to_vec(body).map(|b| BASE64_URL_SAFE_NO_PAD.encode(b))?;
    let payload = [header_b64.as_bytes(), b".", body_b64.as_bytes()].concat();
    let signature = signer.sign(&payload).await;
    let signature_b64 = BASE64_URL_SAFE_NO_PAD.encode(signature);
    Ok(format!("{header_b64}.{body_b64}.{signature_b64}"))
}

#[cfg(test)]
mod tests {
    use openid4vp::verifier::request_signer::P256Signer;
    use p256::elliptic_curve::rand_core::OsRng;
    use x509_cert::certificate::CertificateInner;

    use super::*;

    const CERT_CHAIN_PEM: &[u8] = b"-----BEGIN CERTIFICATE-----
MIICBDCCAaqgAwIBAgIUGgff3n8FETWQOElmhyJgIFCuw7swCgYIKoZIzj0EAwIw
TDELMAkGA1UEBhMCVVMxDjAMBgNVBAgMBVVTLUNBMQ8wDQYDVQQKDAZDQS1ETVYx
HDAaBgNVBAMME0NhbGlmb3JuaWEgRE1WIFRlc3QwHhcNMjYwMzEzMDAxNDA1WhcN
MzYwMzEwMDAxNDA1WjBMMQswCQYDVQQGEwJVUzEOMAwGA1UECAwFVVMtQ0ExDzAN
BgNVBAoMBkNBLURNVjEcMBoGA1UEAwwTQ2FsaWZvcm5pYSBETVYgVGVzdDBZMBMG
ByqGSM49AgEGCCqGSM49AwEHA0IABMZxjYHR7SMNedpoqURMqEkS8eIFI2kNwWiV
F8y2pr7UJJOJoWXqY0x/KCUrbtpiaCtKfHebsP1nuN+D6TbLNvqjajBoMB0GA1Ud
DgQWBBTtKzxtYzVAM/ulLn2fbSAM1EZrujAfBgNVHSMEGDAWgBTtKzxtYzVAM/ul
Ln2fbSAM1EZrujAPBgNVHRMBAf8EBTADAQH/MBUGA1UdEQQOMAyCCmRtdi5jYS5n
b3YwCgYIKoZIzj0EAwIDSAAwRQIgHkb0AlN/biyOZnhsjS2vUoI5rwU22dbGf2tv
UndDlH0CIQD4ibjRKaN7nV1sa34Tr4xjt3BQehQFugXbJ7ajVNETEA==
-----END CERTIFICATE-----";

    #[test]
    fn test_x5chain() {
        let x5c =
            CertificateInner::load_pem_chain(CERT_CHAIN_PEM).expect("failed to load PEM chain");
        assert_eq!(x5c.len(), 1);

        let secret_key = p256::SecretKey::random(&mut OsRng);
        let signer = P256Signer::new(secret_key.into()).expect("failed to create signer");

        let client = X509SanClient::new(x5c, Arc::new(signer), X509SanVariant::Dns)
            .expect("failed to create X509SanClient");
        assert_eq!(client.id.0, "x509_san_dns:dmv.ca.gov");
    }
}
