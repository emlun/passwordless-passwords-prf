use js_sys::Array;
use js_sys::Promise;
use js_sys::Uint8Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::CredentialCreationOptions;
use web_sys::PublicKeyCredential;
use web_sys::PublicKeyCredentialCreationOptions;
use web_sys::PublicKeyCredentialParameters;
use web_sys::PublicKeyCredentialRpEntity;
use web_sys::PublicKeyCredentialType;
use web_sys::PublicKeyCredentialUserEntity;
use yew::function_component;
use yew::html;
use yew::Callback;
use yew::Html;
use yew::Properties;

fn webauthn_create() -> Result<Promise, JsValue> {
    web_sys::window()
        .unwrap()
        .navigator()
        .credentials()
        .create_with_options(CredentialCreationOptions::new().public_key(
            &PublicKeyCredentialCreationOptions::new(
                &Uint8Array::from([0, 1, 2, 3].as_slice()),
                &Array::of1(&PublicKeyCredentialParameters::new(
                    -7,
                    PublicKeyCredentialType::PublicKey,
                )),
                PublicKeyCredentialRpEntity::new("Example app").id("localhost"),
                &PublicKeyCredentialUserEntity::new(
                    "user@example.org",
                    "Example user",
                    &Uint8Array::from([4, 5, 6, 7].as_slice()),
                ),
            ),
        ))
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub on_create: Callback<PublicKeyCredential>,
}

#[function_component]
pub fn CreateButton(props: &Props) -> Html {
    let onclick = {
        let on_create = props.on_create.clone();

        let cb = Closure::new(move |cred: JsValue| {
            console::log_1(&cred);
            let pkcred = PublicKeyCredential::from(cred);
            on_create.emit(pkcred);
        });

        move |_| {
            if let Ok(prom) = webauthn_create() {
                prom.then(&cb);
            } else {
                console::error_1(&"WebAuthn failed".into());
            }
        }
    };

    html! {
        <button {onclick} >{ "Create" }</button>
    }
}
