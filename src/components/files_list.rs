use js_sys::Uint8Array;
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

use crate::crypto::decrypt::decrypt;
use crate::data::vault::VaultConfig;

#[derive(PartialEq, Properties)]
pub struct FileItemProps {
    pub config: Rc<VaultConfig>,
    pub file: String,
}

#[function_component]
pub fn FileItem(props: &FileItemProps) -> Html {
    let decrypted = use_state(|| None);

    let on_hide = Callback::from({
        let decrypted = decrypted.clone();
        move |_| decrypted.set(None)
    });

    let on_show = Callback::from({
        let decrypted = decrypted.clone();
        let config = Rc::clone(&props.config);
        let file = props.file.clone();
        move |_| {
            let decrypted = decrypted.clone();
            let config = Rc::clone(&config);
            let file = file.clone();
            console::log_1(&"spawning...".into());
            spawn_local(async move {
                console::log_1(&"spawned!".into());
                if let Ok(dec) =
                    decrypt(&config.user, &file, config.files.get(&file).unwrap()).await
                {
                    console::log_1(&"Finished!".into());
                    decrypted.set(Some(dec));
                }
                console::log_1(&"Exit.".into());
            });
        }
    });

    html! {
        <div class={classes!("file-item")}>
            <div class={classes!("header")}>
                <pre>{ &props.file }</pre>

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
                                { String::from_utf8(Uint8Array::new(&password).to_vec()).unwrap() }
                            </pre>
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
}

#[function_component]
pub fn FilesList(props: &Props) -> Html {
    let files = props
        .config
        .files
        .keys()
        .map(|name| {
            html! {
                <li key={name.to_string()}>
                    <FileItem
                        config={Rc::clone(&props.config)}
                        file={name.to_string()}
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
