use std::rc::Rc;
use yew::function_component;
use yew::html;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::data::Credential;
use crate::data::CredentialId;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub credentials: Rc<Vec<Credential>>,
    pub on_delete: Callback<CredentialId>,
}

#[function_component]
pub fn CredentialsList(props: &Props) -> Html {
    let credentials = props
        .credentials
        .iter()
        .map(|cred| {
            let on_delete = props.on_delete.clone();
            let cred_raw_id = cred.id.clone();
            let delete = move |_| {
                on_delete.emit(cred_raw_id.clone());
            };

            html! {
                <li key={&cred.id}>
                    { &cred.id.b64 }
                    <button onclick={delete}>{ "Delete" }</button>
                </li>
            }
        })
        .collect::<Html>();

    html! {
        <ul>{credentials}</ul>
    }
}
