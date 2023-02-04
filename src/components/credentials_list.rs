use crate::data::CredentialId;
use std::rc::Rc;
use web_sys::PublicKeyCredential;
use yew::function_component;
use yew::html;
use yew::Callback;
use yew::Html;
use yew::Properties;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub credentials: Rc<Vec<PublicKeyCredential>>,
    pub on_delete: Callback<CredentialId>,
}

#[function_component]
pub fn CredentialsList(props: &Props) -> Html {
    let credentials = props
        .credentials
        .iter()
        .map(|cred| {
            let on_delete = props.on_delete.clone();
            let cred_raw_id = cred.raw_id();
            let delete = move |_| {
                on_delete.emit(CredentialId(cred_raw_id.clone()));
            };

            html! {
                <li key={cred.id()}>
                    { cred.id() }
                    <button onclick={delete}>{ "Delete" }</button>
                </li>
            }
        })
        .collect::<Html>();

    html! {
        <ul>{credentials}</ul>
    }
}
