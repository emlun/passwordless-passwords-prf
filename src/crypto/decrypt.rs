use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Object;
use js_sys::Reflect;
use js_sys::Uint8Array;
use pkcs8::der::AnyRef;
use pkcs8::der::Encode;
use pkcs8::AlgorithmIdentifier;
use pkcs8::ObjectIdentifier;
use pkcs8::PrivateKeyInfo;
use sec1::EcPrivateKey;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;
use web_sys::AesGcmParams;
use web_sys::AesKeyGenParams;
use web_sys::CryptoKey;
use web_sys::EcKeyImportParams;
use web_sys::EcdhKeyDeriveParams;
use web_sys::HkdfParams;
use web_sys::PublicKeyCredential;
use web_sys::SubtleCrypto;

use crate::data::vault::PasswordFile;
use crate::data::vault::UserConfig;
use crate::data::Base64;
use crate::webauthn::prf_extension;
use crate::webauthn::webauthn_get;

pub async fn decrypt(
    vault_config: &UserConfig,
    file_name: &str,
    file_config: &PasswordFile,
) -> Result<ArrayBuffer, JsValue> {
    console::log_1(&"Init".into());

    let subtle: SubtleCrypto = web_sys::window()
        .ok_or(JsValue::UNDEFINED)?
        .crypto()?
        .subtle();

    let cred = JsFuture::from(webauthn_get(
        &[Uint8Array::try_from(&vault_config.fido_credentials[0].id)
            .unwrap()
            .buffer()],
        Some(prf_extension(&vault_config.fido_credentials[0])),
    )?)
    .await?;
    let cred: PublicKeyCredential = cred.dyn_into().unwrap();

    let extensions: Object = cred.get_client_extension_results().dyn_into().unwrap();
    let prf_result_first = Base64::try_from(Uint8Array::new(
        &Reflect::get(
            &Reflect::get(
                &Reflect::get(&extensions, &"prf".into()).unwrap(),
                &"results".into(),
            )
            .unwrap(),
            &"first".into(),
        )
        .unwrap(),
    ))
    .unwrap()
    .0;

    let cred_id = Base64::try_from(cred.raw_id()).unwrap();
    let prf_output = prf_result_first;

    console::log_1(&"PrfEvaluated".into());

    let ecdh: ObjectIdentifier = "1.2.840.10045.2.1".parse().unwrap();
    let named_curve_p256: ObjectIdentifier = "1.2.840.10045.3.1.7".parse().unwrap();
    let pki_vec = PrivateKeyInfo::new(
        AlgorithmIdentifier {
            oid: ecdh,
            parameters: Some(AnyRef::from(&named_curve_p256)),
        },
        &EcPrivateKey {
            private_key: &prf_output,
            parameters: None,
            public_key: None,
        }
        .to_vec()
        .unwrap(),
    )
    .to_vec()
    .unwrap();

    let authnr_private_key: CryptoKey = JsFuture::from(subtle.import_key_with_object(
        "pkcs8",
        &Uint8Array::from(pki_vec.as_slice()),
        EcKeyImportParams::new("ECDH").named_curve("P-256"),
        false,
        &Array::of1(&"deriveKey".into()),
    )?)
    .await?
    .into();

    console::log_1(&"AuthenticatorKeyDerived".into());

    let file_exchange_key: CryptoKey = JsFuture::from(subtle.import_key_with_object(
        "spki",
        &Uint8Array::try_from(&file_config.keys.get(&cred_id).unwrap().ecdh_pubkey).unwrap(),
        EcKeyImportParams::new("ECDH").named_curve("P-256"),
        false,
        &Array::new(),
    )?)
    .await?
    .into();

    console::log_1(&"FileExchangeKeyImported".into());

    let file_wrapping_hkdf_input: CryptoKey =
        JsFuture::from(subtle.derive_key_with_object_and_str(
            &EcdhKeyDeriveParams::new("ECDH", &file_exchange_key),
            &authnr_private_key,
            "HKDF",
            false,
            &Array::of1(&"deriveKey".into()),
        )?)
        .await?
        .into();

    console::log_1(&"FileWrappingHkdfInputImported".into());

    let file_wrapping_key: CryptoKey = JsFuture::from(subtle.derive_key_with_object_and_object(
        &HkdfParams::new(
            "HKDF",
            &"SHA-256".into(),
            &Uint8Array::from(file_name.as_bytes()),
            &Uint8Array::try_from(&file_config.keys.get(&cred_id).unwrap().hkdf_salt).unwrap(),
        ),
        &file_wrapping_hkdf_input,
        &AesKeyGenParams::new("AES-KW", 128),
        false,
        &Array::of2(&"wrapKey".into(), &"unwrapKey".into()),
    )?)
    .await?
    .into();

    console::log_1(&"FileWrappingKeyDerived".into());

    let password_key_encrypted =
        Uint8Array::try_from(&file_config.keys.get(&cred_id).unwrap().wrapped_key).unwrap();

    let file_password_key: CryptoKey =
        JsFuture::from(subtle.unwrap_key_with_buffer_source_and_str_and_str(
            "raw",
            &password_key_encrypted.buffer(),
            &file_wrapping_key,
            "AES-KW",
            "AES-GCM",
            false,
            &Array::of2(&"encrypt".into(), &"decrypt".into()),
        )?)
        .await?
        .into();

    console::log_1(&"FilePasswordKeyUnwrapped".into());

    let password = JsFuture::from(subtle.decrypt_with_object_and_buffer_source(
        &AesGcmParams::new("AES-GCM", &Uint8Array::try_from(&file_config.iv).unwrap()),
        &file_password_key,
        &Uint8Array::try_from(&file_config.content).unwrap(),
    )?)
    .await?;

    console::log_1(&"PasswordDecrypted".into());

    Ok(password.into())
}
