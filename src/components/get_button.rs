use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Promise;
use js_sys::Uint8Array;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::CredentialRequestOptions;
use web_sys::PublicKeyCredential;
use web_sys::PublicKeyCredentialDescriptor;
use web_sys::PublicKeyCredentialRequestOptions;
use web_sys::PublicKeyCredentialType;
use yew::function_component;
use yew::html;
use yew::Html;
use yew::Properties;

fn webauthn_get(ids: &[ArrayBuffer]) -> Result<Promise, JsValue> {
    web_sys::window()
        .unwrap()
        .navigator()
        .credentials()
        .get_with_options(
            CredentialRequestOptions::new().public_key(
                PublicKeyCredentialRequestOptions::new(&Uint8Array::from([0, 1, 2, 3].as_slice()))
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
    pub credentials: Rc<Vec<PublicKeyCredential>>,
}

#[function_component]
pub fn GetButton(props: &Props) -> Html {
    let credids: Vec<ArrayBuffer> = props.credentials.iter().map(|cred| cred.raw_id()).collect();
    let credids_empty = credids.is_empty();

    let onclick = {
        let cb = Closure::new(|cred: JsValue| {
            console::log_1(&cred);
        });

        move |_| {
            if let Ok(prom) = webauthn_get(&credids) {
                prom.then(&cb);
            } else {
                console::error_1(&"WebAuthn failed".into());
            }
        }
    };

    html! {
        <button {onclick} disabled={credids_empty}>{ "Get" }</button>
    }
}
