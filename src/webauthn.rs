use std::rc::Rc;

use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Object;
use js_sys::Promise;
use js_sys::Reflect;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::AuthenticationExtensionsClientInputs;
use web_sys::CredentialCreationOptions;
use web_sys::CredentialRequestOptions;
use web_sys::PublicKeyCredential;
use web_sys::PublicKeyCredentialCreationOptions;
use web_sys::PublicKeyCredentialParameters;
use web_sys::PublicKeyCredentialRequestOptions;
use web_sys::PublicKeyCredentialType;

use crate::crypto::WrappedKeypair;
use crate::crypto::WrappedKeypairAdditionalData;
use crate::data::vault::UserConfig;
use crate::error::JsOrSerdeError;

pub fn webauthn_create(
    challenge: &[u8],
    vault_config: &UserConfig,
    extensions: Option<&AuthenticationExtensionsClientInputs>,
) -> Result<Promise, JsOrSerdeError> {
    Ok(web_sys::window()
        .unwrap()
        .navigator()
        .credentials()
        .create_with_options({
            let mut opt = PublicKeyCredentialCreationOptions::new(
                &Uint8Array::from(challenge),
                &Array::of1(&PublicKeyCredentialParameters::new(
                    -7,
                    PublicKeyCredentialType::PublicKey,
                )),
                &crate::config::webauthn::rp_entity(),
                &vault_config.webauthn_user(),
            );
            opt.exclude_credentials(&vault_config.webauthn_credential_descriptors()?.into());
            if let Some(ext) = extensions {
                opt.extensions(ext);
            }
            CredentialCreationOptions::new().public_key(&opt)
        })?)
}

pub fn webauthn_get(
    challenge: &[u8],
    vault_config: &UserConfig,
    extensions: Option<&AuthenticationExtensionsClientInputs>,
) -> Result<Promise, JsOrSerdeError> {
    Ok(webauthn_get_with_allow_credentials(
        challenge,
        vault_config.webauthn_credential_descriptors()?,
        extensions,
    )?)
}

pub fn webauthn_get_with_allow_credentials(
    challenge: &[u8],
    allow_credentials: Array,
    extensions: Option<&AuthenticationExtensionsClientInputs>,
) -> Result<Promise, JsValue> {
    let mut options = PublicKeyCredentialRequestOptions::new(&Uint8Array::from(challenge))
        .rp_id(crate::config::webauthn::rp_id())
        .allow_credentials(&allow_credentials)
        .to_owned();

    if let Some(extensions) = extensions {
        options.extensions(extensions);
    }

    web_sys::window()
        .unwrap()
        .navigator()
        .credentials()
        .get_with_options(CredentialRequestOptions::new().public_key(&options))
}

pub fn prf_extension_eval(salt: &ArrayBuffer) -> AuthenticationExtensionsClientInputs {
    AuthenticationExtensionsClientInputs::from(
        Object::from_entries(&Array::of1(&Array::of2(
            &"prf".into(),
            &Object::from_entries(&Array::of1(&Array::of2(
                &"eval".into(),
                &Object::from_entries(&Array::of1(&Array::of2(&"first".into(), salt))).unwrap(),
            )))
            .unwrap(),
        )))
        .unwrap()
        .dyn_into::<JsValue>()
        .unwrap(),
    )
}

pub fn prf_extension_eval_by_credential(
    recipients: &[Rc<WrappedKeypair>],
) -> Result<AuthenticationExtensionsClientInputs, JsOrSerdeError> {
    Ok(AuthenticationExtensionsClientInputs::from(
        Object::from_entries(&Array::of1(&Array::of2(
            &"prf".into(),
            &Object::from_entries(
                &Array::of1(&Array::of2(
                    &"evalByCredential".into(),
                    &Object::from_entries(
                        &recipients
                            .into_iter()
                            .map(|wkp| -> Result<Array, JsOrSerdeError> {
                                let ead: WrappedKeypairAdditionalData = wkp.additional_data()?;
                                Ok(Array::of2(
                                    &ead.credential_id().b64url().into(),
                                    &Object::from_entries(&Array::of1(&Array::of2(
                                        &"first".into(),
                                        &Uint8Array::from(ead.prf_salt.as_slice()),
                                    )))?
                                    .into(),
                                ))
                            })
                            .collect::<Result<Array, JsOrSerdeError>>()?
                            .into(),
                    )?
                    .into(),
                ))
                .into(),
            )?
            .into(),
        )))?
        .dyn_into::<JsValue>()?,
    ))
}

pub fn prf_first_output(cred: &PublicKeyCredential) -> Result<Uint8Array, JsValue> {
    let extensions: Object = cred.get_client_extension_results().dyn_into()?;
    let prf_output: Result<Uint8Array, JsValue> = Reflect::get(&extensions, &"prf".into())
        .and_then(|prf| Reflect::get(&prf, &"results".into()))
        .and_then(|prf_results| Reflect::get(&prf_results, &"first".into()))
        .map(|first| Uint8Array::new(&first));
    prf_output
}
