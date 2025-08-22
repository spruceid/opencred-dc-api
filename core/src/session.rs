use isomdl::presentation::authentication::ResponseAuthenticationOutcome;
use serde::{Deserialize, Serialize};

type SessionId = String;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InitiatedSessionState {
    // aka `Annex C`
    OrgIsoMDoc {
        private_key: Vec<u8>,
        session_transcript_bytes: Vec<u8>,
    },
    // aka `Annex D`
    OpenId4VP {
        oid4vp_session_id: String,
        origin: String,
        nonce: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Created(SessionId),
    Initiated(InitiatedSessionState),
    Completed(ResponseAuthenticationOutcome),
}
