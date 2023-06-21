use std::collections::HashSet;
use std::rc::Rc;

use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Reflect;
use js_sys::Uint8Array;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;
use web_sys::AesGcmParams;
use web_sys::AesKeyGenParams;
use web_sys::Crypto;
use web_sys::CryptoKey;
use web_sys::EcKeyGenParams;
use web_sys::EcdhKeyDeriveParams;
use web_sys::HkdfParams;
use web_sys::PublicKeyCredential;
use web_sys::PublicKeyCredentialDescriptor;
use web_sys::PublicKeyCredentialType;
use web_sys::SubtleCrypto;

use crate::data::vault::UserConfig;
use crate::data::CredentialId;
use crate::error::JsOrSerdeError;
use crate::webauthn::prf_extension_eval;
use crate::webauthn::prf_extension_eval_by_credential;
use crate::webauthn::prf_first_output;
use crate::webauthn::webauthn_create;
use crate::webauthn::webauthn_get_with_allow_credentials;

const AES_SIZE: u16 = 256;
const AES_IV_LENGTH: usize = 96 / 8;
const EC_CURVE: &str = "P-256";

type AesIv = [u8; AES_IV_LENGTH];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WrappedKeypair {
    #[serde(with = "crate::data::base64")]
    wrapped_private_key: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    pub iv: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    additional_data: Vec<u8>,
    pub nickname: Option<String>,
}

impl WrappedKeypair {
    pub fn additional_data(&self) -> Result<WrappedKeypairAdditionalData, serde_json::Error> {
        serde_json::from_slice(&self.additional_data)
    }
}

#[derive(Serialize, Deserialize)]
pub struct WrappedKeypairAdditionalData {
    #[serde(with = "crate::data::base64")]
    credential_id: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    pubkey: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    pub prf_salt: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    hkdf_salt: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    hkdf_info: Vec<u8>,
}

impl WrappedKeypairAdditionalData {
    pub fn credential_id(&self) -> CredentialId {
        self.credential_id.clone().into()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WrappedContentKey {
    #[serde(with = "crate::data::base64")]
    pub credential_id: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    wrapping_exchange_pubkey: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    wrapped_content_key: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EncryptedContent {
    #[serde(with = "crate::data::base64")]
    ciphertext: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    iv: Vec<u8>,
    #[serde(with = "crate::data::base64")]
    additional_data: Vec<u8>,
    pub recipients: Vec<WrappedContentKey>,
}

pub fn crypto() -> Result<Crypto, JsValue> {
    web_sys::window().ok_or(JsValue::UNDEFINED)?.crypto()
}

pub fn subtle_crypto() -> Result<SubtleCrypto, JsValue> {
    Ok(web_sys::window()
        .ok_or(JsValue::UNDEFINED)?
        .crypto()?
        .subtle())
}

pub fn gen_random<const N: usize>() -> Result<[u8; N], JsValue> {
    let mut result: [u8; N] = [0; N];
    crypto()?.get_random_values_with_u8_array(&mut result)?;
    Ok(result)
}

pub async fn create_credential(
    vault_config: &UserConfig,
) -> Result<PublicKeyCredential, JsOrSerdeError> {
    console::log_1(&"add_credential 1".into());

    let prf_salt: [u8; 32] = gen_random()?;
    let challenge: [u8; 32] = gen_random()?;

    Ok(JsFuture::from(webauthn_create(
        challenge.as_slice(),
        &vault_config,
        Some(&prf_extension_eval(
            &Uint8Array::from(prf_salt.as_slice()).buffer(),
        )),
    )?)
    .await?
    .into())
}

pub async fn create_wrapped_keypair(
    credential_id: &[u8],
) -> Result<WrappedKeypair, JsOrSerdeError> {
    let subtle = subtle_crypto()?;

    let prf_salt: [u8; 32] = gen_random()?;

    let prf_output: Uint8Array = {
        let get_challenge: [u8; 32] = gen_random()?;
        let cred: PublicKeyCredential = JsFuture::from(webauthn_get_with_allow_credentials(
            get_challenge.as_slice(),
            Some(PublicKeyCredentialDescriptor::new(
                &Uint8Array::from(credential_id),
                PublicKeyCredentialType::PublicKey,
            ))
            .into_iter()
            .collect::<Array>(),
            Some(&prf_extension_eval(
                &Uint8Array::from(prf_salt.as_slice()).buffer(),
            )),
        )?)
        .await?
        .into();
        prf_first_output(&cred)?
    };

    let hkdf_salt: [u8; 32] = gen_random()?;
    let hkdf_info: [u8; 0] = [];
    let base_key: CryptoKey = JsFuture::from(subtle.import_key_with_str(
        "raw",
        &prf_output,
        "HKDF",
        false,
        &Array::of1(&"deriveKey".into()),
    )?)
    .await?
    .into();

    let wrapping_key: CryptoKey = JsFuture::from(subtle.derive_key_with_object_and_object(
        &HkdfParams::new(
            "HKDF",
            &"SHA-256".into(),
            &Uint8Array::from(hkdf_info.as_slice()),
            &Uint8Array::from(hkdf_salt.as_slice()),
        ),
        &base_key,
        &AesKeyGenParams::new("AES-GCM", AES_SIZE),
        false,
        &Array::of1(&"wrapKey".into()),
    )?)
    .await?
    .into();

    let keypair = JsFuture::from(subtle.generate_key_with_object(
        &EcKeyGenParams::new("ECDH", EC_CURVE),
        true,
        &Array::of1(&"deriveKey".into()),
    )?)
    .await?;
    console::log_2(&"keypair".into(), &keypair);

    let pubkey: ArrayBuffer = JsFuture::from(
        subtle.export_key("raw", &Reflect::get(&keypair, &"publicKey".into())?.into())?,
    )
    .await?
    .into();
    console::log_2(&"pubkey".into(), &pubkey);

    let iv: AesIv = gen_random()?;
    let additional_data = WrappedKeypairAdditionalData {
        credential_id: credential_id.into(),
        pubkey: Uint8Array::new(&pubkey).to_vec(),
        prf_salt: prf_salt.into(),
        hkdf_salt: hkdf_salt.into(),
        hkdf_info: hkdf_info.into(),
    };
    let additional_data_bytes: Vec<u8> = serde_json::to_vec(&additional_data)?;

    let wrapped_private_key: ArrayBuffer = JsFuture::from(
        subtle.wrap_key_with_object(
            "jwk",
            &Reflect::get(&keypair, &"privateKey".into())?.into(),
            &wrapping_key,
            AesGcmParams::new("AES-GCM", &Uint8Array::from(iv.as_slice()))
                .additional_data(&Uint8Array::from(additional_data_bytes.as_slice())),
        )?,
    )
    .await?
    .into();
    console::log_2(&"wrapped_private_key".into(), &wrapped_private_key);

    Ok(WrappedKeypair {
        wrapped_private_key: Uint8Array::new(&wrapped_private_key).to_vec(),
        iv: iv.into(),
        additional_data: additional_data_bytes,
        nickname: None,
    })
}

pub async fn unwrap_private_key(
    wrapped_keypairs: &[Rc<WrappedKeypair>],
) -> Result<(Vec<u8>, CryptoKey), JsOrSerdeError> {
    let get_challenge: [u8; 32] = gen_random()?;
    let cred: PublicKeyCredential = JsFuture::from(webauthn_get_with_allow_credentials(
        get_challenge.as_slice(),
        wrapped_keypairs
            .iter()
            .map(|wkp| {
                let ead = wkp.additional_data()?;
                Ok(PublicKeyCredentialDescriptor::new(
                    &Uint8Array::from(ead.credential_id.as_slice()),
                    PublicKeyCredentialType::PublicKey,
                ))
            })
            .collect::<Result<Array, JsOrSerdeError>>()?,
        Some(&prf_extension_eval_by_credential(wrapped_keypairs)?),
    )?)
    .await?
    .into();

    let credential_id: Vec<u8> = Uint8Array::new(&cred.raw_id()).to_vec();
    let prf_output: Uint8Array = prf_first_output(&cred)?;

    // console::log_2(&"credential_id".into(), &credential_id);
    console::log_1(&"prf_output".into());

    let wrapped_keypair: &WrappedKeypair = wrapped_keypairs
        .iter()
        .find(|wkp| {
            wkp.additional_data()
                .map_or(false, |ead| ead.credential_id == credential_id)
        })
        .unwrap();

    let additional_data = wrapped_keypair.additional_data()?;

    let subtle: SubtleCrypto = subtle_crypto()?;
    let base_key: CryptoKey = JsFuture::from(subtle.import_key_with_str(
        "raw",
        &prf_output,
        "HKDF",
        false,
        &Array::of1(&"deriveKey".into()),
    )?)
    .await?
    .into();
    console::log_2(&"base_key".into(), &base_key);

    let wrapping_key: CryptoKey = JsFuture::from(subtle.derive_key_with_object_and_object(
        &HkdfParams::new(
            "HKDF",
            &"SHA-256".into(),
            &Uint8Array::from(additional_data.hkdf_info.as_slice()),
            &Uint8Array::from(additional_data.hkdf_salt.as_slice()),
        ),
        &base_key,
        &AesKeyGenParams::new("AES-GCM", AES_SIZE),
        false,
        &Array::of1(&"unwrapKey".into()),
    )?)
    .await?
    .into();
    console::log_2(&"wrapping_key".into(), &wrapping_key);

    let mut wrapped_private_key: Vec<u8> = wrapped_keypair.wrapped_private_key.to_vec();

    let private_key: CryptoKey = JsFuture::from(
        subtle.unwrap_key_with_u8_array_and_object_and_object(
            "jwk",
            wrapped_private_key.as_mut_slice(),
            &wrapping_key,
            AesGcmParams::new("AES-GCM", &Uint8Array::from(wrapped_keypair.iv.as_slice()))
                .additional_data(&Uint8Array::from(
                    wrapped_keypair.additional_data.as_slice(),
                )),
            &EcKeyGenParams::new("ECDH", EC_CURVE),
            false,
            &Array::of1(&"deriveKey".into()),
        )?,
    )
    .await?
    .into();

    Ok((credential_id, private_key))
}

pub async fn encrypt_content_key_to_recipient(
    content_key: &CryptoKey,
    wrapped_keypair: &WrappedKeypair,
) -> Result<WrappedContentKey, JsOrSerdeError> {
    let subtle: SubtleCrypto = subtle_crypto()?;

    let wrapping_exchange_keypair = JsFuture::from(subtle.generate_key_with_object(
        &EcKeyGenParams::new("ECDH", EC_CURVE),
        false,
        &Array::of1(&"deriveKey".into()),
    )?)
    .await?;
    console::log_2(
        &"wrapping_exchange_keypair".into(),
        &wrapping_exchange_keypair,
    );

    let recipient_pubkey: CryptoKey = JsFuture::from(subtle.import_key_with_object(
        "raw",
        &Uint8Array::from(wrapped_keypair.additional_data()?.pubkey.as_slice()),
        &EcKeyGenParams::new("ECDH", EC_CURVE),
        false,
        &Array::new(), // CHROME: []   FIREFOX: ["deriveKey"]
    )?)
    .await?
    .into();
    console::log_2(&"recipient_pubkey".into(), &recipient_pubkey);

    let wrapping_key: CryptoKey = JsFuture::from(subtle.derive_key_with_object_and_object(
        &EcdhKeyDeriveParams::new("ECDH", &recipient_pubkey),
        &Reflect::get(&wrapping_exchange_keypair, &"privateKey".into())?.into(),
        &AesKeyGenParams::new("AES-KW", AES_SIZE),
        false,
        &Array::of1(&"wrapKey".into()),
    )?)
    .await?
    .into();
    console::log_2(&"wrapping_key".into(), &wrapping_key);

    let wrapped_content_key: ArrayBuffer =
        JsFuture::from(subtle.wrap_key_with_str("raw", &content_key, &wrapping_key, "AES-KW")?)
            .await?
            .into();
    console::log_2(&"wrapped_content_key".into(), &wrapped_content_key);

    let wrapping_exchange_pubkey: ArrayBuffer = JsFuture::from(subtle.export_key(
        "raw",
        &Reflect::get(&wrapping_exchange_keypair, &"publicKey".into())?.into(),
    )?)
    .await?
    .into();
    console::log_2(
        &"wrapping_exchange_pubkey".into(),
        &wrapping_exchange_pubkey,
    );

    Ok(WrappedContentKey {
        credential_id: wrapped_keypair.additional_data()?.credential_id,
        wrapping_exchange_pubkey: Uint8Array::new(&wrapping_exchange_pubkey).to_vec(),
        wrapped_content_key: Uint8Array::new(&wrapped_content_key).to_vec(),
    })
}

pub async fn encrypt(
    mut data: Vec<u8>,
    wrapped_keypairs: &[Rc<WrappedKeypair>],
) -> Result<EncryptedContent, JsOrSerdeError> {
    let subtle: SubtleCrypto = subtle_crypto()?;

    let content_key: CryptoKey = JsFuture::from(subtle.generate_key_with_object(
        &AesKeyGenParams::new("AES-GCM", AES_SIZE),
        true,
        &Array::of1(&"encrypt".into()),
    )?)
    .await?
    .into();

    let mut wrapping_keys: Vec<WrappedContentKey> = Vec::new();
    for wrapped_keypair in wrapped_keypairs {
        wrapping_keys.push(encrypt_content_key_to_recipient(&content_key, wrapped_keypair).await?);
    }

    let iv: AesIv = gen_random()?;
    let additional_data_bytes: Vec<u8> = vec![];

    let ciphertext: ArrayBuffer = JsFuture::from(
        subtle.encrypt_with_object_and_u8_array(
            AesGcmParams::new("AES-GCM", &Uint8Array::from(iv.as_slice()))
                .additional_data(&Uint8Array::from(additional_data_bytes.as_slice())),
            &content_key,
            &mut data,
        )?,
    )
    .await?
    .into();

    Ok(EncryptedContent {
        ciphertext: Uint8Array::new(&ciphertext).to_vec(),
        iv: iv.to_vec(),
        additional_data: additional_data_bytes,
        recipients: wrapping_keys,
    })
}

pub async fn decrypt(
    data: &EncryptedContent,
    wrapped_keypairs: &[Rc<WrappedKeypair>],
) -> Result<Vec<u8>, JsOrSerdeError> {
    let subtle: SubtleCrypto = subtle_crypto()?;

    let valid_credential_ids: HashSet<&Vec<u8>> = data
        .recipients
        .iter()
        .map(|wck| &wck.credential_id)
        .collect();

    let valid_keypairs: Vec<Rc<WrappedKeypair>> = wrapped_keypairs
        .iter()
        .cloned()
        .filter(|wkp| {
            wkp.additional_data()
                .map_or(false, |ad| valid_credential_ids.contains(&ad.credential_id))
        })
        .collect();

    let (credential_id, private_key): (Vec<u8>, CryptoKey) =
        unwrap_private_key(&valid_keypairs).await?;
    console::log_2(
        &"credential_id".into(),
        &Uint8Array::from(credential_id.as_slice()),
    );

    let recipient: &WrappedContentKey = data
        .recipients
        .iter()
        .find(|wkp| wkp.credential_id == credential_id)
        .unwrap();

    let pubkey: CryptoKey = JsFuture::from(subtle.import_key_with_object(
        "raw",
        &Uint8Array::from(recipient.wrapping_exchange_pubkey.as_slice()),
        &EcKeyGenParams::new("ECDH", EC_CURVE),
        false,
        &Array::new(), // CHROME: []   FIREFOX: ["deriveKey"]
    )?)
    .await?
    .into();
    console::log_2(&"pubkey".into(), &pubkey);

    let wrapping_key: CryptoKey = JsFuture::from(subtle.derive_key_with_object_and_object(
        &EcdhKeyDeriveParams::new("ECDH", &pubkey),
        &private_key,
        &AesKeyGenParams::new("AES-KW", AES_SIZE),
        false,
        &Array::of1(&"unwrapKey".into()),
    )?)
    .await?
    .into();
    console::log_2(&"wrapping_key".into(), &wrapping_key);

    let mut wrapped_content_key = recipient.wrapped_content_key.clone();
    let content_key: CryptoKey = JsFuture::from(subtle.unwrap_key_with_u8_array_and_str_and_str(
        "raw",
        &mut wrapped_content_key,
        &wrapping_key,
        "AES-KW",
        "AES-GCM",
        false,
        &Array::of1(&"decrypt".into()),
    )?)
    .await?
    .into();
    console::log_2(&"content_key".into(), &content_key);

    let mut ciphertext = data.ciphertext.clone();
    let content: ArrayBuffer = JsFuture::from(
        subtle.decrypt_with_object_and_u8_array(
            AesGcmParams::new("AES-GCM", &Uint8Array::from(data.iv.as_slice()))
                .additional_data(&Uint8Array::from(data.additional_data.as_slice())),
            &content_key,
            &mut ciphertext,
        )?,
    )
    .await?
    .into();

    Ok(Uint8Array::new(&content).to_vec())
}
