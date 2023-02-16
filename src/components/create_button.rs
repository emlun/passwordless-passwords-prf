use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Promise;
use js_sys::Uint8Array;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::CredentialCreationOptions;
use web_sys::DomException;
use web_sys::PublicKeyCredential;
use web_sys::PublicKeyCredentialCreationOptions;
use web_sys::PublicKeyCredentialDescriptor;
use web_sys::PublicKeyCredentialParameters;
use web_sys::PublicKeyCredentialRpEntity;
use web_sys::PublicKeyCredentialType;
use web_sys::PublicKeyCredentialUserEntity;
use yew::function_component;
use yew::html;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::data::Credential;

fn webauthn_create(credential_ids: &[ArrayBuffer]) -> Result<Promise, JsValue> {
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
                    PublicKeyCredentialRpEntity::new("Example app")
                        .id("tla.app.k8s.dev.yubico.org"),
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

#[derive(PartialEq, Properties)]
pub struct Props {
    pub on_begin: Callback<()>,
    pub on_create: Callback<PublicKeyCredential>,
    pub on_fail: Callback<String>,
    pub credentials: Rc<Vec<Credential>>,
}

#[function_component]
pub fn CreateButton(props: &Props) -> Html {
    let onclick = {
        let on_begin = props.on_begin.clone();
        let on_create = props.on_create.clone();
        let on_fail = props.on_fail.clone();

        let cred_ids: Vec<ArrayBuffer> = props
            .credentials
            .iter()
            .map(|cred| cred.id.raw.clone())
            .collect();

        let cb = Closure::new(move |cred: JsValue| {
            console::log_1(&cred);
            let pkcred = PublicKeyCredential::from(cred);
            on_create.emit(pkcred);
            on_begin.emit(());
        });

        let fail_cb = Closure::new(move |err: JsValue| {
            let domex = DomException::from(err.clone());
            if domex.is_undefined() {
                unimplemented!("{:?}", err);
            } else {
                match domex.code() {
                    DomException::ABORT_ERR => {}
                    DomException::INVALID_STATE_ERR => {
                        on_fail.emit("Authenticator already registered.".to_string());
                    }
                    _ => unimplemented!("{}: {}", domex.code(), domex.name()),
                }
            }
        });

        move |_| {
            if let Ok(prom) = webauthn_create(&cred_ids) {
                let _ = prom.then(&cb).catch(&fail_cb);
            } else {
                console::error_1(&"WebAuthn failed".into());
            }
        }
    };

    html! {
        <button {onclick} >{ "Create" }</button>
    }
}
