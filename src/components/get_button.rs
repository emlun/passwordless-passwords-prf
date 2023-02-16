use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Promise;
use js_sys::Uint8Array;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::CredentialRequestOptions;
use web_sys::DomException;
use web_sys::PublicKeyCredentialDescriptor;
use web_sys::PublicKeyCredentialRequestOptions;
use web_sys::PublicKeyCredentialType;
use yew::function_component;
use yew::html;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::data::Credential;

fn webauthn_get(ids: &[ArrayBuffer]) -> Result<Promise, JsValue> {
    web_sys::window()
        .unwrap()
        .navigator()
        .credentials()
        .get_with_options(
            CredentialRequestOptions::new().public_key(
                PublicKeyCredentialRequestOptions::new(&Uint8Array::from([0, 1, 2, 3].as_slice()))
                    .rp_id("tla.app.k8s.dev.yubico.org")
                    .allow_credentials(
                        &ids.iter()
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
    pub on_fail: Callback<String>,
    pub credentials: Rc<Vec<Credential>>,
}

#[function_component]
pub fn GetButton(props: &Props) -> Html {
    let credids: Vec<ArrayBuffer> = props
        .credentials
        .iter()
        .map(|cred| cred.id.raw.clone())
        .collect();
    let credids_empty = credids.is_empty();

    let onclick = {
        let on_begin = props.on_begin.clone();
        let on_fail = props.on_fail.clone();

        let cb = Closure::new(move |cred: JsValue| {
            console::log_1(&cred);
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
                        on_fail.emit("Authenticator is not registered.".to_string());
                    }
                    _ => match domex.name().as_str() {
                        "NotAllowedError" => {
                            on_fail.emit("Authentication failed.".to_string());
                        }
                        _ => unimplemented!("{}: {}", domex.code(), domex.name()),
                    },
                }
            }
        });

        move |_| {
            if let Ok(prom) = webauthn_get(&credids) {
                let _ = prom.then(&cb).catch(&fail_cb);
            } else {
                console::error_1(&"WebAuthn failed".into());
            }
        }
    };

    html! {
        <button {onclick} disabled={credids_empty}>{ "Get" }</button>
    }
}
