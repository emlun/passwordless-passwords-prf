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
    error: Option<String>,
}

enum AppAction {
    Add(PublicKeyCredential),
    Delete(CredentialId),
    SetError(String),
    ClearError,
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

            Self::Action::SetError(msg) => {
                Rc::make_mut(&mut self).error = Some(msg);
                self
            }

            Self::Action::ClearError => {
                Rc::make_mut(&mut self).error = None;
                self
            }
        }
    }
}

#[function_component]
pub fn App() -> Html {
    let state = use_reducer_eq(AppState::default);
    let credentials = Rc::clone(&state.credentials);

    let on_clear_error = {
        let state = state.clone();
        Callback::from(move |_| {
            state.dispatch(AppAction::ClearError);
        })
    };

    let on_create = {
        let state = state.clone();
        Callback::from(move |cred: PublicKeyCredential| {
            state.dispatch(AppAction::Add(cred));
        })
    };

    let on_set_error = {
        let state = state.clone();
        Callback::from(move |msg| {
            state.dispatch(AppAction::SetError(msg));
        })
    };

    let on_delete = {
        let state = state.clone();
        Callback::from(move |cred_id| {
            state.dispatch(AppAction::Delete(cred_id));
        })
    };

    html! {
        <>
            <div>
                <CreateButton
                    credentials={Rc::clone(&credentials)}
                    {on_create}
                    on_begin={on_clear_error.clone()}
                    on_fail={on_set_error.clone()}
                />
                <GetButton
                    credentials={Rc::clone(&credentials)}
                    on_begin={on_clear_error}
                    on_fail={on_set_error}
                />
                { state.error.as_ref() }
                <CredentialsList {credentials} {on_delete} />
            </div>
        </>
    }
}
