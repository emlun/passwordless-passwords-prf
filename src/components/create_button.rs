use js_sys::Uint8Array;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::function_component;
use yew::html;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::crypto::create_credential;
use crate::crypto::create_wrapped_keypair;
use crate::crypto::WrappedKeypair;
use crate::data::vault::VaultConfig;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub on_begin: Callback<()>,
    pub on_create: Callback<WrappedKeypair>,
    pub on_fail: Callback<String>,
    pub config: Rc<VaultConfig>,
}

#[function_component]
pub fn CreateButton(props: &Props) -> Html {
    let onclick = {
        let config = Rc::clone(&props.config);
        let on_create = props.on_create.clone();
        move |_| {
            let config = Rc::clone(&config);
            let on_create = on_create.clone();
            spawn_local(async move {
                if let Ok(cred) = create_credential(&config.user).await {
                    if let Ok(wrapped_keypair) =
                        create_wrapped_keypair(&Uint8Array::new(&cred.raw_id()).to_vec()).await
                    {
                        console::log_1(&"Finished!".into());
                        on_create.emit(wrapped_keypair);
                    } else {
                        console::log_1(&"Failed to create encryption keypair.".into());
                    }
                } else {
                    console::log_1(&"WebAuthn registration failed.".into());
                }
            });
        }
    };

    html! {
        <button {onclick} >{ "Add key" }</button>
    }
}
