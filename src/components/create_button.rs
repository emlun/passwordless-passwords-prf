use js_sys::ArrayBuffer;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::DomException;
use web_sys::PublicKeyCredential;
use yew::function_component;
use yew::html;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::data::Credential;
use crate::webauthn::webauthn_create;

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
