use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    // List of certificates to include for verification
    certs: Vec<String>,
}
