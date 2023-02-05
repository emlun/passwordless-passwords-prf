use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::HtmlInputElement;
use yew::function_component;
use yew::html;
use yew::use_state;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::data::Credential;
use crate::data::CredentialId;

#[derive(PartialEq, Properties)]
pub struct CredentialItemProps {
    pub credential: Credential,
    pub on_delete: Callback<()>,
    pub on_rename: Callback<String>,
}

#[function_component]
pub fn CredentialItem(props: &CredentialItemProps) -> Html {
    let new_name = use_state(|| props.credential.nickname.clone().unwrap_or_default());
    let editing = use_state(|| false);

    if *editing {
        let current_new_name = (*new_name).clone();

        let on_save_rename = {
            let nn = new_name.clone();
            let ed = editing.clone();
            let on_rename = props.on_rename.clone();
            let name = (*nn).clone();
            move |_| {
                let name = name.trim().to_string();
                ed.set(false);
                nn.set(name.clone());
                on_rename.emit(name);
            }
        };

        let on_change_name = {
            move |e: Event| {
                if let Some(el) = e
                    .target()
                    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                {
                    new_name.set(el.value());
                }
            }
        };

        html! {
            <li>
                <form onsubmit={on_save_rename}>
                    <input
                        type="text"
                        value={ current_new_name }
                        onchange={on_change_name}
                    />
                    <button type="submit">{ "Save" }</button>
                </form>
                <button onclick={move |_| editing.set(false)}>{ "Cancel" }</button>
            </li>
        }
    } else {
        let on_delete = {
            let on_delete = props.on_delete.clone();
            move |_| {
                on_delete.emit(());
            }
        };

        let name = props
            .credential
            .nickname
            .clone()
            .unwrap_or(props.credential.id.b64_abbrev(24));

        html! {
            <li>
                { name }
                <button onclick={move |_| editing.set(true)}>{ "Rename" }</button>
                <button onclick={on_delete}>{ "Delete" }</button>
            </li>
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub credentials: Rc<Vec<Credential>>,
    pub on_delete: Callback<CredentialId>,
    pub on_rename: Callback<(CredentialId, String)>,
}

#[function_component]
pub fn CredentialsList(props: &Props) -> Html {
    let credentials = props
        .credentials
        .iter()
        .map(|cred| {
            let on_delete = props.on_delete.clone();
            let on_rename = props.on_rename.clone();

            let on_delete = {
                let cred_raw_id = cred.id.clone();
                move |_| {
                    on_delete.emit(cred_raw_id.clone());
                }
            };
            let on_rename = {
                let cred_raw_id = cred.id.clone();
                move |name| {
                    on_rename.emit((cred_raw_id.clone(), name));
                }
            };

            let key = cred.id.clone();

            html! {
                <CredentialItem
                    {key}
                    credential={cred.clone()}
                    {on_delete}
                    {on_rename}
                />
            }
        })
        .collect::<Html>();

    html! {
        <ul>{credentials}</ul>
    }
}
