use isomdl::definitions::helpers::{NonEmptyMap, NonEmptyVec};
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use tsify::Tsify;

use crate::{annex_c, annex_d};

// Note this is also referred to as `Annex C`,
// in reference to ISO/IEC 18013-7
#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct DCAPIRequestOrgIsoMDoc {
    pub device_request: String,
    pub encryption_info: String,
}

// NOTE: This is also referred to as `Annex D`,
// in reference to ISO/IEC 18013-7.
#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DCAPIRequestOpenId4VP {
    pub request: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase", tag = "protocol")]
pub enum DCAPIRequest {
    #[serde(rename = "org-iso-mdoc")]
    OrgIsoMDoc { data: DCAPIRequestOrgIsoMDoc },
    #[serde(rename = "openid4vp")]
    OpenId4VP { data: DCAPIRequestOpenId4VP },
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct DCAPINamespaceRequest {
    // NonEmptyMap and NonEmptyVec come from isomdl and serialize as a plain
    // map/array; override the TS type so consumers see the on-the-wire shape.
    #[cfg_attr(feature = "wasm", tsify(type = "Record<string, string[]>"))]
    pub namespaces: NonEmptyMap<String, NonEmptyVec<String>>,
    pub origin: String,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase", tag = "protocol")]
pub enum DCAPIResponse {
    #[serde(rename = "org-iso-mdoc")]
    OrgIsoMDoc { data: annex_c::MDocResponseData },
    #[serde(rename = "openid4vp")]
    OpenId4VP {
        data: annex_d::OpenId4VPResponseData,
    },
}
