use js_sys::Array;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::PublicKeyCredentialDescriptor;
use web_sys::PublicKeyCredentialUserEntity;

use crate::crypto::encrypt;
use crate::crypto::EncryptedContent;
use crate::crypto::WrappedKeypair;
use crate::error::JsOrSerdeError;

use super::CredentialId;
use super::UserHandle;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VaultConfig {
    #[serde(rename = "v")]
    version: u32,

    pub user: Rc<UserConfig>,
    pub contents: HashMap<String, Rc<EncryptedContent>>,
}

impl VaultConfig {
    pub async fn new(username: String) -> Result<Self, JsValue> {
        Ok(Self {
            version: 2,
            user: Rc::new(UserConfig::new(username).await?),
            contents: HashMap::new(),
        })
    }

    pub fn rename_credential(
        &mut self,
        cred_id: &CredentialId,
        name: String,
    ) -> Result<&mut Self, JsOrSerdeError> {
        for mut keypair in Rc::make_mut(&mut Rc::make_mut(&mut self.user).keypairs).iter_mut() {
            if keypair.additional_data()?.credential_id() == *cred_id {
                Rc::make_mut(&mut keypair).nickname = Some(name);
                return Ok(self);
            }
        }
        Err(JsOrSerdeError::JsError("Credential not found".into()))
    }

    pub fn delete_credential(&mut self, cred_id: &CredentialId) -> &mut Self {
        Rc::make_mut(&mut Rc::make_mut(&mut self.user).keypairs).retain(|wkp| {
            !wkp.additional_data()
                .map_or(false, |ad| ad.credential_id() == *cred_id)
        });

        for (_, mut content) in &mut self.contents {
            Rc::make_mut(&mut content)
                .recipients
                .retain(|wkk| CredentialId::from(wkk.credential_id.clone()) != *cred_id)
        }

        self
    }

    pub async fn push_content(
        &mut self,
        name: String,
        content: Vec<u8>,
    ) -> Result<&mut Self, JsOrSerdeError> {
        let encrypted = encrypt(content, self.user.keypairs.as_slice()).await?;
        self.contents.insert(name, Rc::new(encrypted));
        Ok(self)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserConfig {
    #[serde(rename = "v")]
    pub version: u32,

    pub username: String,
    pub user_handle: UserHandle,
    pub keypairs: Rc<Vec<Rc<WrappedKeypair>>>,
}

impl UserConfig {
    pub async fn new(username: String) -> Result<Self, JsValue> {
        Ok(Self {
            version: 2,
            username,
            user_handle: UserHandle::generate().await?,
            keypairs: Rc::new(Vec::new()),
        })
    }

    pub fn webauthn_user(&self) -> PublicKeyCredentialUserEntity {
        PublicKeyCredentialUserEntity::new(
            &self.username,
            &self.username,
            &self.user_handle.uint8_array(),
        )
    }

    pub fn webauthn_credential_descriptors(&self) -> Result<Array, JsOrSerdeError> {
        self.keypairs
            .iter()
            .map(|wkp| {
                Ok(PublicKeyCredentialDescriptor::from(
                    &wkp.additional_data()?.credential_id(),
                ))
            })
            .collect()
    }
}
