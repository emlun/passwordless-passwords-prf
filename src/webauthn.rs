use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Object;
use js_sys::Promise;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::AuthenticationExtensionsClientInputs;
use web_sys::CredentialCreationOptions;
use web_sys::CredentialRequestOptions;
use web_sys::PublicKeyCredentialCreationOptions;
use web_sys::PublicKeyCredentialDescriptor;
use web_sys::PublicKeyCredentialParameters;
use web_sys::PublicKeyCredentialRequestOptions;
use web_sys::PublicKeyCredentialType;
use web_sys::PublicKeyCredentialUserEntity;

use crate::data::vault::FidoCredential;

pub fn webauthn_create(credential_ids: &[ArrayBuffer]) -> Result<Promise, JsValue> {
    web_sys::window()
        .unwrap()
        .navigator()
        .credentials()
        .create_with_options(
            CredentialCreationOptions::new().public_key(
                PublicKeyCredentialCreationOptions::new(
                    &Uint8Array::from([0, 1, 2, 3].as_slice()),
                    &Array::of1(&PublicKeyCredentialParameters::new(
                        -7,
                        PublicKeyCredentialType::PublicKey,
                    )),
                    &crate::config::webauthn::rp_entity(),
                    &PublicKeyCredentialUserEntity::new(
                        "user@example.org",
                        "Example user",
                        &Uint8Array::from([4, 5, 6, 7].as_slice()),
                    ),
                )
                .exclude_credentials(
                    &credential_ids
                        .iter()
                        .map(|id| {
                            PublicKeyCredentialDescriptor::new(
                                id,
                                PublicKeyCredentialType::PublicKey,
                            )
                        })
                        .collect::<Array>(),
                ),
            ),
        )
}

pub fn webauthn_get(
    ids: &[ArrayBuffer],
    extensions: Option<AuthenticationExtensionsClientInputs>,
) -> Result<Promise, JsValue> {
    let mut options =
        PublicKeyCredentialRequestOptions::new(&Uint8Array::from([0, 1, 2, 3].as_slice()))
            .rp_id(crate::config::webauthn::rp_id())
            .allow_credentials(
                &ids.iter()
                    .map(|id| {
                        PublicKeyCredentialDescriptor::new(id, PublicKeyCredentialType::PublicKey)
                    })
                    .collect::<Array>(),
            )
            .to_owned();

    if let Some(extensions) = extensions {
        options.extensions(&extensions);
    }

    web_sys::window()
        .unwrap()
        .navigator()
        .credentials()
        .get_with_options(CredentialRequestOptions::new().public_key(&options))
}

pub fn prf_extension(cred: &FidoCredential) -> AuthenticationExtensionsClientInputs {
    AuthenticationExtensionsClientInputs::from(
        Object::from_entries(&Array::of1(&Array::of2(
            &"prf".into(),
            &Object::from_entries(&Array::of1(&Array::of2(
                &"eval".into(),
                &Object::from_entries(&Array::of1(&Array::of2(
                    &"first".into(),
                    &Uint8Array::try_from(&cred.prf_salt).unwrap().buffer(),
                )))
                .unwrap(),
            )))
            .unwrap(),
        )))
        .unwrap()
        .dyn_into::<JsValue>()
        .unwrap(),
    )
}
