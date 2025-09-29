use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use openid4vp::{
    core::authorization_request::{
        AuthorizationRequestObject,
        parameters::{ClientId, ClientIdScheme, ResponseMode},
    },
    verifier::{client::X509SanVariant, request_signer::P256Signer, session::SessionStore},
};
use pkcs8::DecodePrivateKey;

use super::{config::Oid4VpConfig, x509_client::X509SanClient};

#[derive(Debug, Clone)]
pub struct OID4VPClient(X509SanClient);

impl OID4VPClient {
    pub fn new(config: &Oid4VpConfig) -> Result<Self> {
        let secret_key =
            p256::SecretKey::from_pkcs8_pem(&config.client.key).context("Could not load JWK")?;
        let signer = P256Signer::new(secret_key.into())?;
        let inner = X509SanClient::new(
            config.client.x5c.clone(),
            Arc::new(signer),
            X509SanVariant::Dns,
        )
        .context("Could not build OID4VP client")?;
        Ok(Self(inner))
    }
}

#[cfg_attr(target_arch="wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl openid4vp::verifier::client::Client for OID4VPClient {
    fn id(&self) -> &ClientId {
        self.0.id()
    }

    fn scheme(&self) -> ClientIdScheme {
        self.0.scheme()
    }

    async fn generate_request_object_jwt(
        &self,
        body: &AuthorizationRequestObject,
    ) -> Result<String> {
        self.0.generate_request_object_jwt(body).await
    }
}

#[derive(Debug, Clone)]
pub struct OID4VPVerifier(pub openid4vp::verifier::Verifier);

impl OID4VPVerifier {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new(
        config: &Oid4VpConfig,
        oid4vp_client: OID4VPClient,
        session_store: Arc<dyn SessionStore + Send + Sync>,
    ) -> Result<Self> {
        Self::new_internal(config, oid4vp_client, session_store).await
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new(
        config: &Oid4VpConfig,
        oid4vp_client: OID4VPClient,
        session_store: Arc<dyn SessionStore>,
    ) -> Result<Self> {
        Self::new_internal(config, oid4vp_client, session_store).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn new_internal(
        config: &Oid4VpConfig,
        oid4vp_client: OID4VPClient,
        session_store: Arc<dyn SessionStore + Send + Sync>,
    ) -> Result<Self> {
        let oid4vp_verifier = openid4vp::verifier::Verifier::builder()
            .with_client(Arc::new(oid4vp_client))
            .with_session_store(session_store)
            .with_default_request_parameter(ResponseMode::DirectPost)
            .with_submission_endpoint(
                config
                    .base_url
                    .clone()
                    .join(&config.submission_endpoint)
                    .context("Could not join submission url")?,
            )
            .by_reference(
                config
                    .base_url
                    .join(&config.reference_endpoint)
                    .context("Could not join reference url")?,
            )
            .build()
            .await
            .context("Could not build OID4VP verifier")?;
        Ok(Self(oid4vp_verifier))
    }

    #[cfg(target_arch = "wasm32")]
    async fn new_internal(
        config: &Oid4VpConfig,
        oid4vp_client: OID4VPClient,
        session_store: Arc<dyn SessionStore>,
    ) -> Result<Self> {
        let oid4vp_verifier = openid4vp::verifier::Verifier::builder()
            .with_client(Arc::new(oid4vp_client))
            .with_session_store(session_store)
            .with_default_request_parameter(ResponseMode::DirectPost)
            .with_submission_endpoint(
                config
                    .base_url
                    .clone()
                    .join(&config.submission_endpoint)
                    .context("Could not join submission url")?,
            )
            .by_reference(
                config
                    .base_url
                    .join(&config.reference_endpoint)
                    .context("Could not join reference url")?,
            )
            .build()
            .await
            .context("Could not build OID4VP verifier")?;
        Ok(Self(oid4vp_verifier))
    }
}
