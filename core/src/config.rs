use anyhow::Context;
use serde::{Deserialize, de};
use url::Url;
use x509_cert::{Certificate, certificate::CertificateInner};

#[derive(Deserialize, Debug, Clone)]
pub struct Oid4VpConfig {
    #[serde(alias = "baseurl")]
    pub base_url: Url,
    pub client: Client,
    pub submission_endpoint: String,
    pub reference_endpoint: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Client {
    /// certs chain in PEM format
    #[serde(deserialize_with = "deserialize_x5c")]
    pub x5c: Vec<CertificateInner>,
    /// PEM encoded
    pub key: String,
}

fn deserialize_x5c<'de, D>(deserializer: D) -> Result<Vec<CertificateInner>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Certificate::load_pem_chain(s.as_bytes())
        .context("Could not load x5c")
        .map_err(de::Error::custom)
}
