use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use super::Base64;
use super::UserHandle;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct UserConfig {
    #[serde(rename = "v")]
    version: u32,

    pub user_handle: UserHandle,

    pub fido_credentials: Vec<FidoCredential>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct FidoCredential {
    pub id: Base64,
    pub name: Option<String>,
    pub prf_salt: Base64,
    pub public_key: Base64,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct PasswordFile {
    pub content: Base64,
    pub iv: Base64,

    pub keys: HashMap<Base64, PasswordKey>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct PasswordKey(Base64, Base64, Base64);

impl PasswordKey {
    pub fn exchange_pubkey(&self) -> &Base64 {
        &self.0
    }

    pub fn salt(&self) -> &Base64 {
        &self.1
    }

    pub fn password_key(&self) -> &Base64 {
        &self.2
    }
}
