use serde::{Deserialize, Serialize};

// Note this is also referred to as `Annex C`,
// in reference to ISO/IEC 18013-7
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DCAPIResponseOrgIsoMDoc {
    device_request: String,
    encryption_info: String,
}

// NOTE: This is also referred to as `Annex D`,
// in reference to ISO/IEC 18013-7.
#[derive(Clone, Serialize, Deserialize)]
pub struct DCAPIResponseOpenId4VP {
    pub request: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "protocol")]
pub enum DCAPIResponse {
    #[serde(rename = "org-iso-mdoc")]
    OrgIsoMDoc { data: DCAPIResponseOrgIsoMDoc },
    #[serde(rename = "openid4vp")]
    OpenId4VP { data: DCAPIResponseOpenId4VP },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DCAPIResponses {
    pub requests: Vec<DCAPIResponse>,
}
