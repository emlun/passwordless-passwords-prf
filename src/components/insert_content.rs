use std::rc::Rc;

use stylist::yew::styled_component;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::HtmlInputElement;
use web_sys::HtmlTextAreaElement;
use web_sys::SubmitEvent;
use yew::html;
use yew::use_state;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::data::vault::VaultConfig;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub config: Rc<VaultConfig>,
    pub on_submit: Callback<(String, Vec<u8>)>,
}

#[styled_component]
pub fn InsertContent(props: &Props) -> Html {
    let name = use_state(|| "".to_string());
    let content = use_state(|| "".to_string());

    let on_submit = {
        let name = name.clone();
        let content = content.clone();
        let on_submit = props.on_submit.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            on_submit.emit((name.trim().to_string(), Vec::from((*content).clone())));
            name.set("".to_string());
            content.set("".to_string());
        }
    };

    let on_change_name = {
        let name = name.clone();
        move |e: Event| {
            if let Some(el) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                name.set(el.value());
            }
        }
    };

    let on_change_content = {
        let content = content.clone();
        move |e: Event| {
            if let Some(el) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
            {
                content.set(el.value());
            }
        }
    };

    if props.config.user.keypairs.is_empty() {
        html! {
            <></>
        }
    } else {
        html! {
            <form onsubmit={on_submit}>
                <h2>{ "Add new vault entry" }</h2>
                <div>
                    <p>{ "Name:" }</p>
                    <input
                        type="text"
                        value={ (*name).clone() }
                        onchange={on_change_name}
                    />
                </div>
                <div>
                    <p>{ "Content:" }</p>
                    <textarea
                        value={ (*content).clone() }
                        onchange={on_change_content}
                    />
                </div>
                <div>
                    <button type="submit">{ "Encrypt" }</button>
                </div>
            </form>
        }
    }
}
