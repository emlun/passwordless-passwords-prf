use js_sys::Array;
use js_sys::Map;
use js_sys::Object;
use js_sys::Promise;
use js_sys::Uint8Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::CredentialCreationOptions;
use web_sys::PublicKeyCredentialCreationOptions;
use web_sys::PublicKeyCredentialRpEntity;
use web_sys::PublicKeyCredentialUserEntity;
use yew::function_component;
use yew::html;
use yew::use_state;
use yew::Html;

fn webauthn_create() -> Result<Promise, JsValue> {
    let pkcp = Object::from_entries(
        &Map::new()
            .set(&JsValue::from("type"), &JsValue::from("public-key"))
            .set(&JsValue::from("alg"), &JsValue::from(-7)),
    )?;

    console::log_1(&pkcp);

    web_sys::window()
        .unwrap()
        .navigator()
        .credentials()
        .create_with_options(CredentialCreationOptions::new().public_key(
            &PublicKeyCredentialCreationOptions::new(
                &Uint8Array::from([0, 1, 2, 3].as_slice()),
                &Array::of1(&pkcp),
                PublicKeyCredentialRpEntity::new("Example app").id("localhost"),
                &PublicKeyCredentialUserEntity::new(
                    "user@example.org",
                    "Example user",
                    &Uint8Array::from([4, 5, 6, 7].as_slice()),
                ),
            ),
        ))
}

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let on_create_success = use_state(|| {
        Closure::new(|cred: JsValue| {
            console::log_1(&cred);
        })
    });

    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 20;
            counter.set(value);
            if let Ok(prom) = webauthn_create() {
                prom.then(&on_create_success);
            } else {
                console::error_1(&"WebAuthn failed".into());
            }
        }
    };

    html! {
        <div>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
