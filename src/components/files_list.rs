use std::rc::Rc;
use yew::classes;
use yew::function_component;
use yew::html;
use yew::use_state;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::components::decrypt::Decrypt;
use crate::data::vault::PasswordFile;
use crate::data::vault::VaultConfig;

#[derive(PartialEq, Properties)]
pub struct FileItemProps {
    pub config: Rc<VaultConfig>,
    pub name: String,
    pub file: Rc<PasswordFile>,
}

#[function_component]
pub fn FileItem(props: &FileItemProps) -> Html {
    let decrypted = use_state(|| false);

    let on_hide = Callback::from({
        let decrypted = decrypted.clone();
        move |_| decrypted.set(false)
    });
    let on_show = Callback::from({
        let decrypted = decrypted.clone();
        move |_| decrypted.set(true)
    });

    html! {
        <div class={classes!("file-item")}>
            <div class={classes!("header")}>
                <pre>{ &props.name }</pre>

                {
                    if *decrypted {
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

            <div class={classes!("content", Some("expanded").filter(|_| *decrypted))}>
                {
                    if *decrypted {
                        html! {
                            <pre>
                                <Decrypt
                                config={Rc::clone(&props.config)}
                            file={props.name.clone()}
                            />
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
        .iter()
        .map(|(name, file)| {
            html! {
                <li key={name.to_string()}>
                    <FileItem
                        config={Rc::clone(&props.config)}
                        name={name.to_string()}
                        file={Rc::clone(file)}
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
