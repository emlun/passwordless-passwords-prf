use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::rc::Rc;

use super::Base64;
use super::UserHandle;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct VaultConfig {
    #[serde(rename = "v")]
    version: u32,

    pub user: UserConfig,
    pub files: HashMap<String, Rc<PasswordFile>>,
}

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
    pub ecdh_pubkey: Base64,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct PasswordFile {
    pub content: Base64,
    pub iv: Base64,

    pub keys: HashMap<Base64, PasswordKey>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct PasswordKey {
    pub ecdh_pubkey: Base64,
    pub hkdf_salt: Base64,
    pub wrapped_key: Base64,
}
