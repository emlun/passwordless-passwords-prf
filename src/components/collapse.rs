use stylist::yew::styled_component;
use yew::classes;
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
    pub start_expanded: bool,
    pub button_text: String,
}

#[styled_component]
pub fn Collapse(props: &Props) -> Html {
    let expanded = use_state(|| props.start_expanded);

    let on_toggle = {
        let expanded = expanded.clone();
        Callback::from(move |_| {
            expanded.set(!*expanded);
        })
    };

    let class_expanded = Some("expanded").filter(|_| *expanded);

    html! {
        <div class={classes!("collapse", class_expanded)}>
            <button
                class={classes!("toggle")}
                onclick={on_toggle}
            >
                <span class={classes!("toggle-text")}>
                    { &props.button_text }
                </span>
                <span class={classes!("toggle-icon")}/>
            </button>
            <div class={classes!("content", class_expanded)}>
                { for props.children.iter() }
            </div>
        </div>
    }
}
