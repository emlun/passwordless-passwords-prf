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
use yew::use_reducer_eq;
use yew::use_state;
use yew::Callback;
use yew::Html;
use yew::Reducible;
use yew::UseStateHandle;

use crate::components::create_button::CreateButton;
use crate::components::credentials_list::CredentialsList;
use crate::components::get_button::GetButton;
use crate::data::vault::PasswordFile;
use crate::data::vault::UserConfig;
use crate::data::Base64;
use crate::data::Credential;
use crate::data::CredentialId;
use crate::webauthn::prf_extension;
use crate::webauthn::webauthn_get;

#[derive(Clone, Default, PartialEq)]
struct AppState {
    credentials: Rc<Vec<Credential>>,
    error: Option<String>,
}

enum AppAction {
    Add(PublicKeyCredential),
    Delete(CredentialId),
    SetError(String),
    ClearError,
    Rename(CredentialId, String),
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Add(cred) => {
                Rc::make_mut(&mut Rc::make_mut(&mut self).credentials).push(cred.into());
                self
            }

            Self::Action::Delete(cred_id) => {
                Rc::make_mut(&mut Rc::make_mut(&mut self).credentials).retain(|c| c.id != cred_id);
                self
            }

            Self::Action::SetError(msg) => {
                Rc::make_mut(&mut self).error = Some(msg);
                self
            }

            Self::Action::ClearError => {
                Rc::make_mut(&mut self).error = None;
                self
            }

            Self::Action::Rename(cred_id, name) => {
                for cred in Rc::make_mut(&mut Rc::make_mut(&mut self).credentials) {
                    if cred.id == cred_id {
                        cred.nickname = if name.is_empty() { None } else { Some(name) };
                        break;
                    }
                }

                self
            }
        }
    }
}

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
                console::log_2(&"AuthenticatorKeyDerived".into(), authnr_private_key);

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
                            &Uint8Array::from("foo".as_bytes()),
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
                console::log_2(&"FileWrappingKeyDerived".into(), file_wrapping_key);

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
                console::log_2(&"FilePasswordKeyUnwrapped".into(), file_password_key);

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

            Self::PasswordDecrypted { password } => {
                console::log_2(&"PasswordDecrypted".into(), password);

                Ok(())
            }
        }
    }
}

#[styled_component]
pub fn App() -> Html {
    let state = use_reducer_eq(AppState::default);
    let credentials = Rc::clone(&state.credentials);

    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

    let vault_config: UserConfig = serde_json::from_str(
        &local_storage
            .get_item("cli_vault-user.json")
            .unwrap()
            .unwrap(),
    )
    .unwrap();

    let vault_foo: PasswordFile = serde_json::from_str(
        &local_storage
            .get_item("cli_vault/foo.vlt")
            .unwrap()
            .unwrap(),
    )
    .unwrap();

    let closure_registry: Rc<RefCell<ClosuresMap>> = use_mut_ref(HashMap::new);
    console::log_2(
        &"Closures:".into(),
        &(closure_registry.borrow().len()).into(),
    );

    let procedure_state = use_state(|| DecryptionProcedureState::Init);
    let _ = DecryptionProcedureState::advance(
        &procedure_state,
        &closure_registry,
        &vault_config,
        &vault_foo,
    );

    let on_clear_error = {
        let state = state.clone();
        Callback::from(move |_| {
            state.dispatch(AppAction::ClearError);
        })
    };

    let on_create = {
        let state = state.clone();
        Callback::from(move |cred: PublicKeyCredential| {
            state.dispatch(AppAction::Add(cred));
        })
    };

    let on_set_error = {
        let state = state.clone();
        Callback::from(move |msg| {
            state.dispatch(AppAction::SetError(msg));
        })
    };

    let on_delete = {
        let state = state.clone();
        Callback::from(move |cred_id| {
            state.dispatch(AppAction::Delete(cred_id));
        })
    };

    let on_rename = {
        let state = state.clone();
        Callback::from(move |(cred_id, name)| {
            state.dispatch(AppAction::Rename(cred_id, name));
        })
    };

    html! {
        <div class={css! {
            background: ${"#101010"};
            color: ${"#f1f1f1"};
            display: flex;
            flex-direction: column;
            justify-content: flex-start;
            margin: 0;
            min-height: 100%;
            min-width: 100%;
            padding: 0;
            position: absolute;
        }}>

            <div class={css! {
                flex-grow: 1;
                flex-shrink: 0;
                margin: 0 auto;
                padding: ${"2em 10em"};
            }}>
                <div>
                    <CreateButton
                        credentials={Rc::clone(&credentials)}
                        {on_create}
                        on_begin={on_clear_error.clone()}
                        on_fail={on_set_error.clone()}
                    />
                    <GetButton
                        credentials={Rc::clone(&credentials)}
                        on_begin={on_clear_error}
                        on_fail={on_set_error}
                    />
                    { state.error.as_ref() }
                </div>
                <div>
                    <CredentialsList {credentials} {on_delete} {on_rename} />
                </div>

                <div>
                    {
                        if let DecryptionProcedureState::PasswordDecrypted { password  } = &*procedure_state {
                            html! {
                                <>
                                { "Decrypted password:" }
                                <pre>
                                { String::from_utf8(Uint8Array::new(password).to_vec()).unwrap() }
                                </pre>
                                    </>
                            }
                        } else {
                            html! {
                                <></>
                            }
                        }
                    }
                </div>
            </div>

            <div class={css! {
                border-top: ${"1px solid #626262"};
                color: ${"1px solid #626262"};
                flex-grow: 0;
                flex-shrink: 0;
                padding: ${"1em 10em"};
                text-align: center;
            }}>
                {"Footer"}
            </div>
        </div>
    }
}
