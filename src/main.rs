use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Promise;
use js_sys::Uint8Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::AuthenticatorAssertionResponse;
use web_sys::AuthenticatorAttestationResponse;
use web_sys::CredentialCreationOptions;
use web_sys::CredentialRequestOptions;
use web_sys::PublicKeyCredential;
use web_sys::PublicKeyCredentialCreationOptions;
use web_sys::PublicKeyCredentialDescriptor;
use web_sys::PublicKeyCredentialParameters;
use web_sys::PublicKeyCredentialRequestOptions;
use web_sys::PublicKeyCredentialRpEntity;
use web_sys::PublicKeyCredentialType;
use web_sys::PublicKeyCredentialUserEntity;
use yew::function_component;
use yew::html;
use yew::use_state;
use yew::Html;

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

fn webauthn_get(id: &ArrayBuffer) -> Result<Promise, JsValue> {
    web_sys::window()
        .unwrap()
        .navigator()
        .credentials()
        .get_with_options(
            CredentialRequestOptions::new().public_key(
                PublicKeyCredentialRequestOptions::new(&Uint8Array::from([0, 1, 2, 3].as_slice()))
                    .allow_credentials(&Array::of1(&PublicKeyCredentialDescriptor::new(
                        id,
                        PublicKeyCredentialType::PublicKey,
                    ))),
            ),
        )
}

enum CredentialState {
    NotCreated,
    Created(PublicKeyCredential),
}

#[function_component]
fn WebauthnButtons() -> Html {
    let state = use_state(|| CredentialState::NotCreated);

    let credid = match &*state {
        CredentialState::NotCreated => None,
        CredentialState::Created(credential) => Some(credential.raw_id()),
    };

    let cb = Closure::once(move |cred: JsValue| {
        console::log_1(&cred);
        let pkcred = PublicKeyCredential::from(cred);
        console::log_3(
            &"attestation_object".into(),
            &pkcred
                .response()
                .has_own_property(&"attestationObject".into())
                .into(),
            &AuthenticatorAttestationResponse::from(JsValue::from(pkcred.response()))
                .attestation_object(),
        );
        console::log_2(
            &"authenticator_data".into(),
            &AuthenticatorAssertionResponse::from(JsValue::from(pkcred.response()))
                .authenticator_data(),
        );
        state.set(CredentialState::Created(pkcred));
    });

    let (onclick, onclick_get) = if let Some(credid) = credid {
        (
            None,
            Some(move |_| {
                if let Ok(prom) = webauthn_get(&credid) {
                    prom.then(&cb);
                } else {
                    console::error_1(&"WebAuthn failed".into());
                }
            }),
        )
    } else {
        (
            Some({
                move |_| {
                    if let Ok(prom) = webauthn_create() {
                        prom.then(&cb);
                    } else {
                        console::error_1(&"WebAuthn failed".into());
                    }
                }
            }),
            None,
        )
    };

    html! {
        <div>
            <button {onclick} disabled={onclick.is_none()}>{ "Create" }</button>
            <button onclick={onclick_get} disabled={onclick_get.is_none()}>{ "Get" }</button>
        </div>
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <WebauthnButtons />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
