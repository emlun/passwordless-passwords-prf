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
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use stylist::yew::styled_component;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::AesGcmParams;
use web_sys::AesKeyGenParams;
use web_sys::CryptoKey;
use web_sys::EcKeyImportParams;
use web_sys::EcdhKeyDeriveParams;
use web_sys::HkdfParams;
use web_sys::PublicKeyCredential;
use web_sys::SubtleCrypto;
use yew::html;
use yew::use_mut_ref;
use yew::use_state;
use yew::Html;
use yew::Properties;
use yew::UseStateHandle;

use crate::data::vault::PasswordFile;
use crate::data::vault::UserConfig;
use crate::data::vault::VaultConfig;
use crate::data::Base64;
use crate::webauthn::prf_extension;
use crate::webauthn::webauthn_get;

fn subtle_crypto() -> Result<SubtleCrypto, JsValue> {
    Ok(web_sys::window()
        .ok_or(JsValue::UNDEFINED)?
        .crypto()?
        .subtle())
}

type ClosuresMap<'id> = HashMap<&'id str, Rc<Closure<dyn FnMut(JsValue)>>>;
fn new_closure<'id>(
    closure_registry: &Rc<RefCell<ClosuresMap<'id>>>,
    id: &'id str,
    closure: Closure<dyn FnMut(JsValue)>,
) -> Rc<Closure<dyn FnMut(JsValue)>> {
    let cb = Rc::new(closure);
    closure_registry.borrow_mut().insert(id, Rc::clone(&cb));
    cb
}

enum DecryptionProcedureState {
    Init,
    VaultPubkeyImported(CryptoKey),
    PrfEvaluated {
        cred_id: Base64,
        prf_output: Vec<u8>,
    },
    AuthenticatorKeyDerived {
        cred_id: Base64,
        authnr_private_key: CryptoKey,
    },
    FileExchangeKeyImported {
        cred_id: Base64,
        authnr_private_key: CryptoKey,
        file_exchange_key: CryptoKey,
    },
    FileWrappingHkdfInputImported {
        cred_id: Base64,
        file_wrapping_hkdf_input: CryptoKey,
    },
    FileWrappingKeyDerived {
        cred_id: Base64,
        file_wrapping_key: CryptoKey,
    },
    FilePasswordKeyUnwrapped {
        file_password_key: CryptoKey,
    },
    PasswordDecrypted {
        password: ArrayBuffer,
    },
}

impl DecryptionProcedureState {
    fn advance(
        state: &UseStateHandle<Self>,
        closure_registry: &Rc<RefCell<ClosuresMap>>,
        vault_config: &UserConfig,
        file_name: &str,
        file_config: &PasswordFile,
    ) -> Result<(), JsValue> {
        match &**state {
            Self::Init => {
                console::log_1(&"Init".into());

                let _ = subtle_crypto()?
                    .import_key_with_object(
                        "spki",
                        &Uint8Array::try_from(&vault_config.fido_credentials[0].public_key)
                            .unwrap(),
                        EcKeyImportParams::new("ECDH").named_curve("P-256"),
                        false,
                        &Array::new(),
                    )?
                    .then(&new_closure(closure_registry, "cb1", {
                        let state = state.clone();
                        Closure::new(move |key: JsValue| {
                            state.set(Self::VaultPubkeyImported(key.into()));
                        })
                    }));

                Ok(())
            }

            Self::VaultPubkeyImported(..) => {
                console::log_1(&"VaultPubkeyImported".into());

                let _ = webauthn_get(
                    &[Uint8Array::try_from(&vault_config.fido_credentials[0].id)
                        .unwrap()
                        .buffer()],
                    Some(prf_extension(&vault_config.fido_credentials[0])),
                )?
                .then(&new_closure(closure_registry, "cb2", {
                    let state = state.clone();

                    Closure::new(move |cred: JsValue| {
                        let cred: PublicKeyCredential = cred.dyn_into().unwrap();
                        let extensions: Object =
                            cred.get_client_extension_results().dyn_into().unwrap();
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

                        state.set(Self::PrfEvaluated {
                            cred_id: Base64::try_from(cred.raw_id()).unwrap(),
                            prf_output: prf_result_first,
                        });
                    })
                }));

                Ok(())
            }

            Self::PrfEvaluated {
                cred_id,
                prf_output,
            } => {
                console::log_1(&"PrfEvaluated".into());

                let ecdh: ObjectIdentifier = "1.2.840.10045.2.1".parse().unwrap();
                let named_curve_p256: ObjectIdentifier = "1.2.840.10045.3.1.7".parse().unwrap();
                let pki_vec = PrivateKeyInfo::new(
                    AlgorithmIdentifier {
                        oid: ecdh,
                        parameters: Some(AnyRef::from(&named_curve_p256)),
                    },
                    &EcPrivateKey {
                        private_key: prf_output,
                        parameters: None,
                        public_key: None,
                    }
                    .to_vec()
                    .unwrap(),
                )
                .to_vec()
                .unwrap();

                let _ = subtle_crypto()?
                    .import_key_with_object(
                        "pkcs8",
                        &Uint8Array::from(pki_vec.as_slice()),
                        EcKeyImportParams::new("ECDH").named_curve("P-256"),
                        false,
                        &Array::of1(&"deriveKey".into()),
                    )?
                    .then(&new_closure(closure_registry, "cb3", {
                        let state = state.clone();
                        let cred_id = cred_id.clone();
                        Closure::new(move |key: JsValue| {
                            state.set(Self::AuthenticatorKeyDerived {
                                cred_id: cred_id.clone(),
                                authnr_private_key: key.into(),
                            });
                        })
                    }));

                Ok(())
            }

            Self::AuthenticatorKeyDerived {
                cred_id,
                authnr_private_key,
            } => {
                console::log_1(&"AuthenticatorKeyDerived".into());

                let _ = subtle_crypto()?
                    .import_key_with_object(
                        "spki",
                        &Uint8Array::try_from(
                            file_config.keys.get(cred_id).unwrap().exchange_pubkey(),
                        )
                        .unwrap(),
                        EcKeyImportParams::new("ECDH").named_curve("P-256"),
                        false,
                        &Array::new(),
                    )
                    .unwrap()
                    .then(&new_closure(closure_registry, "cb4", {
                        let state = state.clone();
                        let cred_id = cred_id.clone();
                        let authnr_private_key = authnr_private_key.clone();
                        Closure::new(move |key: JsValue| {
                            state.set(Self::FileExchangeKeyImported {
                                cred_id: cred_id.clone(),
                                authnr_private_key: authnr_private_key.clone(),
                                file_exchange_key: key.into(),
                            });
                        })
                    }));

                Ok(())
            }

            Self::FileExchangeKeyImported {
                cred_id,
                authnr_private_key,
                file_exchange_key,
            } => {
                console::log_1(&"FileExchangeKeyImported".into());

                let _ = subtle_crypto()?
                    .derive_key_with_object_and_str(
                        &EcdhKeyDeriveParams::new("ECDH", file_exchange_key),
                        authnr_private_key,
                        "HKDF",
                        false,
                        &Array::of1(&"deriveKey".into()),
                    )
                    .unwrap()
                    .then(&new_closure(closure_registry, "cb5", {
                        let state = state.clone();
                        let cred_id = cred_id.clone();
                        Closure::new(move |key: JsValue| {
                            state.set(Self::FileWrappingHkdfInputImported {
                                cred_id: cred_id.clone(),
                                file_wrapping_hkdf_input: key.into(),
                            });
                        })
                    }));

                Ok(())
            }

            Self::FileWrappingHkdfInputImported {
                cred_id,
                file_wrapping_hkdf_input,
            } => {
                console::log_1(&"FileWrappingHkdfInputImported".into());

                let _ = subtle_crypto()?
                    .derive_key_with_object_and_object(
                        &HkdfParams::new(
                            "HKDF",
                            &"SHA-256".into(),
                            &Uint8Array::from(file_name.as_bytes()),
                            &Uint8Array::try_from(file_config.keys.get(cred_id).unwrap().salt())
                                .unwrap(),
                        ),
                        file_wrapping_hkdf_input,
                        &AesKeyGenParams::new("AES-KW", 128),
                        false,
                        &Array::of2(&"wrapKey".into(), &"unwrapKey".into()),
                    )
                    .unwrap()
                    .then(&new_closure(closure_registry, "cb6", {
                        let state = state.clone();
                        let cred_id = cred_id.clone();
                        Closure::new(move |key: JsValue| {
                            state.set(Self::FileWrappingKeyDerived {
                                cred_id: cred_id.clone(),
                                file_wrapping_key: key.into(),
                            });
                        })
                    }));

                Ok(())
            }

            Self::FileWrappingKeyDerived {
                cred_id,
                file_wrapping_key,
            } => {
                console::log_1(&"FileWrappingKeyDerived".into());

                let password_key_encrypted =
                    Uint8Array::try_from(file_config.keys.get(cred_id).unwrap().password_key())
                        .unwrap();

                let _ = subtle_crypto()?
                    .unwrap_key_with_buffer_source_and_str_and_str(
                        "raw",
                        &password_key_encrypted.buffer(),
                        file_wrapping_key,
                        "AES-KW",
                        "AES-GCM",
                        false,
                        &Array::of2(&"encrypt".into(), &"decrypt".into()),
                    )
                    .unwrap()
                    .then(&new_closure(closure_registry, "cb7", {
                        let state = state.clone();
                        Closure::new(move |key: JsValue| {
                            state.set(Self::FilePasswordKeyUnwrapped {
                                file_password_key: key.into(),
                            });
                        })
                    }));

                Ok(())
            }

            Self::FilePasswordKeyUnwrapped { file_password_key } => {
                console::log_1(&"FilePasswordKeyUnwrapped".into());

                let _ = subtle_crypto()?
                    .decrypt_with_object_and_buffer_source(
                        &AesGcmParams::new(
                            "AES-GCM",
                            &Uint8Array::try_from(&file_config.iv).unwrap(),
                        ),
                        file_password_key,
                        &Uint8Array::try_from(&file_config.content).unwrap(),
                    )
                    .unwrap()
                    .then(&new_closure(closure_registry, "cb8", {
                        let state = state.clone();
                        Closure::new(move |output: JsValue| {
                            state.set(Self::PasswordDecrypted {
                                password: output.into(),
                            });
                        })
                    }));

                Ok(())
            }

            Self::PasswordDecrypted { .. } => {
                console::log_1(&"PasswordDecrypted".into());

                Ok(())
            }
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub config: Rc<VaultConfig>,
    pub file: String,
}

#[styled_component]
pub fn Decrypt(props: &Props) -> Html {
    let closure_registry: Rc<RefCell<ClosuresMap>> = use_mut_ref(HashMap::new);
    console::log_2(
        &"Closures:".into(),
        &(closure_registry.borrow().len()).into(),
    );

    let procedure_state = use_state(|| DecryptionProcedureState::Init);
    let _ = DecryptionProcedureState::advance(
        &procedure_state,
        &closure_registry,
        &props.config.user,
        &props.file,
        props.config.files.get(&props.file).unwrap(),
    );

    html! {
        <div>
            {
                if let DecryptionProcedureState::PasswordDecrypted { password  } = &*procedure_state {
                    html! {
                        { String::from_utf8(Uint8Array::new(password).to_vec()).unwrap() }
                    }
                } else {
                    html! {
                        <></>
                    }
                }
            }
        </div>
    }
}
