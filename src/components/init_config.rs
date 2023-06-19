use stylist::yew::styled_component;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::HtmlInputElement;
use yew::html;
use yew::use_state;
use yew::Callback;
use yew::Html;
use yew::Properties;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub on_submit: Callback<String>,
}

#[styled_component]
pub fn InitConfig(props: &Props) -> Html {
    let username = use_state(|| "".to_string());

    let on_submit = {
        let username = username.clone();
        let on_submit = props.on_submit.clone();
        move |_| {
            on_submit.emit(username.trim().to_string());
        }
    };

    let on_change = {
        let username = username.clone();
        move |e: Event| {
            if let Some(el) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                username.set(el.value());
            }
        }
    };

    html! {
        <form onsubmit={on_submit}>
            <p>{ "Username:" }</p>
            <input
                type="text"
                value={ (*username).clone() }
                onchange={on_change}
            />
            <button type="submit">{ "Initialize" }</button>
        </form>
    }
}
