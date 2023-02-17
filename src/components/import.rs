use stylist::yew::styled_component;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::HtmlTextAreaElement;
use web_sys::SubmitEvent;
use yew::html;
use yew::use_state;
use yew::Callback;
use yew::Children;
use yew::Html;
use yew::Properties;

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
    pub on_import: Callback<String>,
}

#[styled_component]
pub fn Import(props: &Props) -> Html {
    let value = use_state(String::new);

    let onchange = {
        let value = value.clone();
        Callback::from(move |e: Event| {
            if let Some(el) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
            {
                value.set(el.value());
            }
        })
    };

    let onsubmit = {
        let on_import = props.on_import.clone();
        let value = value.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            on_import.emit((*value).clone());
        })
    };

    html! {
        <form {onsubmit} class={css! {
            display: flex;
            align-items: start;
            flex-direction: column;
            flex-wrap: nowrap;
            margin: ${"1em 0"};
        }}>
            { for props.children.iter() }
            <textarea {onchange} value={( *value ).clone()} />
            <button type="submit">
                { "Import" }
            </button>
        </form>
    }
}
