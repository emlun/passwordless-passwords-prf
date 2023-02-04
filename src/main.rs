use std::rc::Rc;
use web_sys::PublicKeyCredential;
use yew::function_component;
use yew::html;
use yew::use_reducer;
use yew::Callback;
use yew::Html;
use yew::Reducible;

use passwordless_passwords_prf::components::create_button::CreateButton;
use passwordless_passwords_prf::components::get_button::GetButton;

#[derive(Clone, Default, PartialEq)]
struct WebauthnButtonsState {
    credentials: Rc<Vec<PublicKeyCredential>>,
}

enum WebauthnButtonsAction {
    Add(PublicKeyCredential),
}

impl Reducible for WebauthnButtonsState {
    type Action = WebauthnButtonsAction;

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
fn App() -> Html {
    let state = use_reducer(WebauthnButtonsState::default);
    let credentials = Rc::clone(&state.credentials);
    let on_create = Callback::from(move |cred: PublicKeyCredential| {
        state.dispatch(WebauthnButtonsAction::Add(cred));
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

fn main() {
    yew::Renderer::<App>::new().render();
}
