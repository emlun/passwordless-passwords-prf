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

use crate::crypto::WrappedKeypair;
use crate::data::CredentialId;

#[derive(PartialEq, Properties)]
pub struct CredentialItemProps {
    pub keypair: Rc<WrappedKeypair>,
    pub on_delete: Callback<CredentialId>,
    pub on_rename: Callback<(CredentialId, String)>,
}

#[function_component]
pub fn CredentialItem(props: &CredentialItemProps) -> Html {
    let new_name = use_state(|| props.keypair.nickname.clone().unwrap_or_default());
    let editing = use_state(|| false);

    if let Ok(additional_data) = props.keypair.additional_data() {
        let cred_id = additional_data.credential_id();

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
                    on_rename.emit((cred_id.clone(), name));
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
                let cred_id = cred_id.clone();
                move |_| {
                    on_delete.emit(cred_id.clone());
                }
            };

            let name = props
                .keypair
                .nickname
                .clone()
                .unwrap_or(cred_id.b64_abbrev(24));

            html! {
                <li>
                    { name }
                    <button onclick={move |_| editing.set(true)}>{ "Rename" }</button>
                    <button onclick={on_delete}>{ "Delete" }</button>
                </li>
            }
        }
    } else {
        html! {
            <li>
                { "Failed to decode credential." }
            </li>
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub keypairs: Rc<Vec<Rc<WrappedKeypair>>>,
    pub on_delete: Callback<CredentialId>,
    pub on_rename: Callback<(CredentialId, String)>,
}

#[function_component]
pub fn CredentialsList(props: &Props) -> Html {
    let keypairs = props
        .keypairs
        .iter()
        .map(|cred| {
            let on_delete = props.on_delete.clone();
            let on_rename = props.on_rename.clone();

            if let Ok(additional_data) = cred.additional_data() {
                let key = additional_data.credential_id();
                html! {
                    <CredentialItem
                        {key}
                        keypair={Rc::clone(cred)}
                        {on_delete}
                        {on_rename}
                    />
                }
            } else {
                html! {
                    { "Failed to decode credential." }
                }
            }
        })
        .collect::<Html>();

    html! {
        <ul>{keypairs}</ul>
    }
}
