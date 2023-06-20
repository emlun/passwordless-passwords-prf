use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::classes;
use yew::function_component;
use yew::html;
use yew::use_state;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::crypto::decrypt;
use crate::crypto::EncryptedContent;
use crate::data::vault::VaultConfig;
use crate::data::CredentialId;
use crate::error::JsOrSerdeError;

#[derive(PartialEq, Properties)]
pub struct FileItemProps {
    pub config: Rc<VaultConfig>,
    pub name: String,
    pub item: Rc<EncryptedContent>,
    pub on_reencrypt: Callback<(String, Vec<u8>)>,
}

#[function_component]
pub fn FileItem(props: &FileItemProps) -> Html {
    let decrypted = use_state(|| None);
    let show_keys = use_state(|| false);

    let on_hide = Callback::from({
        let decrypted = decrypted.clone();
        move |_| decrypted.set(None)
    });

    let on_show = Callback::from({
        let decrypted = decrypted.clone();
        let config = Rc::clone(&props.config);
        let item = props.item.clone();
        let on_reencrypt = props.on_reencrypt.clone();
        let name = props.name.clone();
        move |_| {
            let decrypted = decrypted.clone();
            let config = Rc::clone(&config);
            let item = item.clone();
            let on_reencrypt = on_reencrypt.clone();
            let name = name.clone();
            spawn_local(async move {
                match decrypt(&item, &config.user.keypairs).await {
                    Ok(dec) => {
                        console::log_1(&"Finished!".into());
                        decrypted.set(Some(String::from_utf8(dec.clone()).unwrap()));
                        on_reencrypt.emit((name, dec));
                    }
                    Err(JsOrSerdeError::JsError(e)) => {
                        console::log_2(&"Decryption failed:".into(), &e);
                    }
                    Err(JsOrSerdeError::SerializeError(e)) => {
                        console::log_2(
                            &"Decryption failed: (De)serialization failed".into(),
                            &e.to_string().into(),
                        );
                    }
                }
            });
        }
    });

    let on_toggle_keys = Callback::from({
        let show_keys = show_keys.clone();
        move |_| {
            show_keys.set(!*show_keys);
        }
    });

    html! {
        <div class={classes!("file-item")}>
            <div class={classes!("header")}>
                <pre>{ &props.name }</pre>

                <button onclick={on_toggle_keys}>
                    { "Keys: " }
                    { props.item.recipients.len() }
                </button>

                {
                    if decrypted.is_some() {
                        html! {
                            <button onclick={on_hide}>
                            { "Hide" }
                            </button>
                        }
                    } else {
                        html! {
                            <button onclick={on_show}>
                            { "Show" }
                            </button>
                        }
                    }
                }
            </div>

            <div class={classes!("content", Some("expanded").filter(|_| decrypted.is_some()))}>
                {
                    if let Some(password) = &*decrypted {
                        html! {
                            <pre>
                                { password }
                            </pre>
                        }
                    } else {
                        html! {
                            <></>
                        }
                    }
                }

                {
                    if *show_keys {
                        html! {
                            <>
                                <p>
                                    { "Encrypted to " }
                                    { props.item.recipients.len() }
                                    { " keys:" }
                                </p>
                                <ul>
                                    {
                                        props.item.recipients.iter()
                                            .map(|wkp| {
                                                let cred_id = CredentialId::from(wkp.credential_id.clone());
                                                let name: String = props.config.get_credential_nickname(&cred_id)
                                                    .map(|s| s.to_string())
                                                    .unwrap_or_else(|| cred_id.b64_abbrev(24));
                                                html! {
                                                    <li key={cred_id.b64url()}>
                                                        { name }
                                                    </li>
                                                }
                                            })
                                            .collect::<Html>()
                                    }
                                </ul>
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
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub config: Rc<VaultConfig>,
    pub on_reencrypt: Callback<(String, Vec<u8>)>,
}

#[function_component]
pub fn FilesList(props: &Props) -> Html {
    let files = props
        .config
        .contents
        .iter()
        .map(|(name, item)| {
            html! {
                <li key={name.to_string()}>
                    <FileItem
                        config={Rc::clone(&props.config)}
                        name={name.clone()}
                        item={item}
                        on_reencrypt={props.on_reencrypt.clone()}
                    />
                </li>
            }
        })
        .collect::<Html>();

    html! {
        <>
            <h2>{ "Vault entries" }</h2>
            <ul class={classes!("files-list")}>{files}</ul>
        </>
    }
}
