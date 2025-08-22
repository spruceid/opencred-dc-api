use isomdl::definitions::helpers::{NonEmptyMap, NonEmptyVec};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum DCAPIRequestType {
    OrgIsoMDoc,
    OpenId4VP,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DCAPIRequest {
    pub namespaces: NonEmptyMap<String, NonEmptyVec<String>>,
    // Not using Url, mainly to avoid an extraneous `/` during serialization.
    pub origin: String,
}
