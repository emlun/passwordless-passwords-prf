use std::rc::Rc;
use web_sys::PublicKeyCredential;
use yew::function_component;
use yew::html;
use yew::Html;
use yew::Properties;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub credentials: Rc<Vec<PublicKeyCredential>>,
}

#[function_component]
pub fn CredentialsList(props: &Props) -> Html {
    html! {
        <div>
            <ul>
                {props.credentials.iter().map(|cred| {
                    html! {
                        <li key={cred.id()}>{ cred.id() }</li>
                    }
                }).collect::<Html>()}
            </ul>
        </div>
    }
}
