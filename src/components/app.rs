use std::rc::Rc;
use web_sys::PublicKeyCredential;
use yew::function_component;
use yew::html;
use yew::use_reducer;
use yew::Callback;
use yew::Html;
use yew::Reducible;

use crate::components::create_button::CreateButton;
use crate::components::get_button::GetButton;

#[derive(Clone, Default, PartialEq)]
struct AppState {
    credentials: Rc<Vec<PublicKeyCredential>>,
}

enum AppAction {
    Add(PublicKeyCredential),
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Add(cred) => {
                Rc::make_mut(&mut Rc::make_mut(&mut self).credentials).push(cred);
                self
            }
        }
    }
}

#[function_component]
pub fn App() -> Html {
    let state = use_reducer(AppState::default);
    let credentials = Rc::clone(&state.credentials);
    let on_create = Callback::from(move |cred: PublicKeyCredential| {
        state.dispatch(AppAction::Add(cred));
    });

    html! {
        <>
            <div>
                <CreateButton {on_create} />
                <GetButton credentials={Rc::clone(&credentials)} />
            </div>
        </>
    }
}
