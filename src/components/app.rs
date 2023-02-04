use std::rc::Rc;
use web_sys::PublicKeyCredential;
use yew::function_component;
use yew::html;
use yew::use_reducer_eq;
use yew::Callback;
use yew::Html;
use yew::Reducible;

use crate::components::create_button::CreateButton;
use crate::components::credentials_list::CredentialsList;
use crate::components::get_button::GetButton;
use crate::data::CredentialId;

#[derive(Clone, Default, PartialEq)]
struct AppState {
    credentials: Rc<Vec<PublicKeyCredential>>,
}

enum AppAction {
    Add(PublicKeyCredential),
    Delete(CredentialId),
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Add(cred) => {
                Rc::make_mut(&mut Rc::make_mut(&mut self).credentials).push(cred);
                self
            }

            Self::Action::Delete(CredentialId(cred_id)) => {
                Rc::make_mut(&mut Rc::make_mut(&mut self).credentials)
                    .retain(|c| c.raw_id() != cred_id);
                self
            }
        }
    }
}

#[function_component]
pub fn App() -> Html {
    let state = use_reducer_eq(AppState::default);
    let credentials = Rc::clone(&state.credentials);

    let on_create = {
        let state = state.clone();
        Callback::from(move |cred: PublicKeyCredential| {
            state.dispatch(AppAction::Add(cred));
        })
    };
    let on_delete = Callback::from(move |cred_id| {
        state.dispatch(AppAction::Delete(cred_id));
    });

    html! {
        <>
            <div>
                <CreateButton {on_create} />
                <GetButton credentials={Rc::clone(&credentials)} />
                <CredentialsList {credentials} {on_delete} />
            </div>
        </>
    }
}
