use isomdl::definitions::helpers::{NonEmptyMap, NonEmptyVec};
use serde::{Deserialize, Serialize};

use crate::{annex_c, annex_d};

// Note this is also referred to as `Annex C`,
// in reference to ISO/IEC 18013-7
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DCAPIRequestOrgIsoMDoc {
    pub device_request: String,
    pub encryption_info: String,
}

// NOTE: This is also referred to as `Annex D`,
// in reference to ISO/IEC 18013-7.
#[derive(Clone, Serialize, Deserialize)]
pub struct DCAPIRequestOpenId4VP {
    pub request: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "protocol")]
pub enum DCAPIRequest {
    #[serde(rename = "org-iso-mdoc")]
    OrgIsoMDoc { data: DCAPIRequestOrgIsoMDoc },
    #[serde(rename = "openid4vp")]
    OpenId4VP { data: DCAPIRequestOpenId4VP },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DCAPIRequests {
    pub requests: Vec<DCAPIRequest>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DCAPIRequestType {
    OrgIsoMDoc,
    OpenId4VP,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DCAPINamespaceRequest {
    pub namespaces: NonEmptyMap<String, NonEmptyVec<String>>,
    pub origin: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "protocol")]
pub enum DCAPIResponse {
    #[serde(rename = "org-iso-mdoc")]
    OrgIsoMDoc { data: annex_c::DCAPIResponseData },
    #[serde(rename = "openid4vp")]
    OpenId4VP { data: annex_d::DCAPIResponseData },
}
