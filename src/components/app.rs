use std::rc::Rc;
use stylist::yew::styled_component;
use web_sys::console;
use web_sys::PublicKeyCredential;
use yew::html;
use yew::use_reducer_eq;
use yew::Callback;
use yew::Html;
use yew::Reducible;

use crate::components::create_button::CreateButton;
use crate::components::credentials_list::CredentialsList;
use crate::components::decrypt::Decrypt;
use crate::components::get_button::GetButton;
use crate::components::import::Import;
use crate::data::vault::VaultConfig;
use crate::data::Credential;
use crate::data::CredentialId;
use crate::hooks::local_storage::use_local_storage;
use crate::hooks::local_storage::UseLocalStorageHandle;

#[derive(Clone, Default, PartialEq)]
struct AppState {
    credentials: Rc<Vec<Credential>>,
    error: Option<String>,
}

enum AppAction {
    Add(PublicKeyCredential),
    Delete(CredentialId),
    SetError(String),
    ClearError,
    Rename(CredentialId, String),
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::Add(cred) => {
                Rc::make_mut(&mut Rc::make_mut(&mut self).credentials).push(cred.into());
                self
            }

            Self::Action::Delete(cred_id) => {
                Rc::make_mut(&mut Rc::make_mut(&mut self).credentials).retain(|c| c.id != cred_id);
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

            Self::Action::Rename(cred_id, name) => {
                for cred in Rc::make_mut(&mut Rc::make_mut(&mut self).credentials) {
                    if cred.id == cred_id {
                        cred.nickname = if name.is_empty() { None } else { Some(name) };
                        break;
                    }
                }

                self
            }
        }
    }
}

#[styled_component]
pub fn App() -> Html {
    let state = use_reducer_eq(AppState::default);
    let credentials = Rc::clone(&state.credentials);

    let config: UseLocalStorageHandle<VaultConfig> = use_local_storage("vault").unwrap();

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

    let on_rename = {
        let state = state.clone();
        Callback::from(move |(cred_id, name)| {
            state.dispatch(AppAction::Rename(cred_id, name));
        })
    };

    let on_import_config = {
        let config = config.clone();
        Callback::from(move |s: String| {
            if let Err(err) = config.set_from_str(&s) {
                console::error_2(&"Import failed".into(), &err.to_string().into());
            }
        })
    };

    html! {
        <div class={css! {
            background: ${"#101010"};
            color: ${"#f1f1f1"};
            display: flex;
            flex-direction: column;
            justify-content: flex-start;
            margin: 0;
            min-height: 100%;
            min-width: 100%;
            padding: 0;
            position: absolute;
        }}>

            <div class={css! {
                flex-grow: 1;
                flex-shrink: 0;
                margin: 0 auto;
                padding: ${"2em 10em"};
            }}>
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
                </div>
                <div>
                    <CredentialsList {credentials} {on_delete} {on_rename} />
                </div>

                {
                    if let Some(config) = config.ok() {
                        html! {
                            <Decrypt
                                {config}
                                file={"foo".to_string()}
                            />
                        }
                    } else {
                        html! {
                            <>
                                <Import on_import={on_import_config} ><h2>{ "Import vault config:" }</h2></Import>
                            </>
                        }
                    }
                }
            </div>

            <div class={css! {
                border-top: ${"1px solid #626262"};
                color: ${"1px solid #626262"};
                flex-grow: 0;
                flex-shrink: 0;
                padding: ${"1em 10em"};
                text-align: center;
            }}>
                {"Footer"}
            </div>
        </div>
    }
}
