use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD};
use isomdl::presentation::authentication::ResponseAuthenticationOutcome;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use uuid::Uuid;

#[cfg_attr(target_arch="wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait DcApiSessionEngine {
    async fn new_session(&self, session_id: String, session: Session) -> Result<()>;
    async fn get_session(&self, id: String, client_secret: String) -> Result<Option<Session>>;
    async fn get_session_unauthenticated(&self, id: String) -> Result<Option<Session>>;
    async fn update_session(&self, session_id: String, session: Session) -> Result<()>;
    async fn remove_session(&self, session_id: String) -> Result<()>;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub client_secret_hash: String,
    pub state: SessionState,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionCreation {
    pub session: Session,
    pub session_creation_response: SessionCreationResponse,
}

impl Session {
    pub fn new_with_creation_response() -> Result<SessionCreation> {
        let session_id = Uuid::new_v4().to_string();
        let client_secret = Uuid::new_v4().to_string();
        let client_secret_hash = Sha512::digest(client_secret.as_bytes());
        let session_creation_response = SessionCreationResponse {
            id: session_id.clone(),
            client_secret,
        };
        let session = Session {
            client_secret_hash: BASE64_STANDARD.encode(client_secret_hash),
            state: SessionState::Created,
        };

        Ok(SessionCreation {
            session,
            session_creation_response,
        })
    }

    pub fn check_client_secret(client_secret: &str, hash: &str) -> Result<()> {
        let client_secret_hash = Sha512::digest(client_secret.as_bytes());
        let session_client_secret_hash = BASE64_STANDARD
            .decode(hash)
            .map_err(|e| anyhow!("client secret check failed: {e:?}"))?;

        if session_client_secret_hash != client_secret_hash.to_vec() {
            return Err(anyhow!("client secret mismatch"));
        }

        Ok(())
    }
}

pub struct SessionStorage(Arc<dyn DcApiSessionEngine>);

impl SessionStorage {
    pub fn new(engine: Arc<dyn DcApiSessionEngine>) -> Self {
        Self(engine)
    }

    pub async fn new_session(&self) -> Result<SessionCreationResponse> {
        let SessionCreation {
            session,
            session_creation_response,
        } = Session::new_with_creation_response()?;

        self.0
            .new_session(session_creation_response.id.clone(), session)
            .await?;

        Ok(session_creation_response)
    }

    pub async fn get_session(&self, id: String, client_secret: String) -> Result<Option<Session>> {
        let session = self.0.get_session(id, client_secret.clone()).await?;
        let session = if let Some(s) = session {
            s
        } else {
            return Ok(None);
        };
        let client_secret_hash = Sha512::digest(client_secret.as_bytes());
        // Returning Ok(None) to avoid leaking information
        let session_client_secret_hash =
            match BASE64_STANDARD.decode(session.client_secret_hash.clone()) {
                Ok(h) => h,
                Err(e) => {
                    tracing::debug!("Couldn't decode base64 client secret: {e:?}");
                    return Ok(None);
                }
            };
        if session_client_secret_hash != client_secret_hash.to_vec() {
            tracing::debug!("Client secret mismatch");
            return Ok(None);
        }
        Ok(Some(session))
    }

    pub async fn get_session_unauthenticated(&self, id: String) -> Result<Option<Session>> {
        let session = self.0.get_session_unauthenticated(id).await?;
        let session = if let Some(s) = session {
            s
        } else {
            return Ok(None);
        };
        Ok(Some(session))
    }

    pub async fn update_session(&self, session_id: String, session: Session) -> Result<()> {
        self.0.update_session(session_id, session).await
    }

    pub async fn remove_session(&self, session_id: String) -> Result<()> {
        self.0.remove_session(session_id).await
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionCreationResponse {
    id: String,
    client_secret: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Created,
    Initiated {
        annex_c: super::annex_c::InitiatedSessionState,
        annex_d: super::annex_d::InitiatedSessionState,
    },
    Completed(ResponseAuthenticationOutcome),
}
