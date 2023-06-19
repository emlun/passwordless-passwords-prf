use std::rc::Rc;
use stylist::yew::styled_component;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::classes;
use yew::html;
use yew::use_reducer_eq;
use yew::Callback;
use yew::Html;
use yew::Reducible;

use crate::components::collapse::Collapse;
use crate::components::import::Import;
use crate::components::init_config::InitConfig;
use crate::components::vault::Vault;
use crate::data::vault::VaultConfig;
use crate::hooks::local_storage::use_local_storage;
use crate::hooks::local_storage::UseLocalStorageHandle;

#[derive(Clone, Default, PartialEq)]
struct AppState {
    error: Option<String>,
}

enum AppAction {
    JsOrSerdeError(String),
    ClearError,
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::JsOrSerdeError(msg) => {
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

#[styled_component]
pub fn App() -> Html {
    let state = use_reducer_eq(AppState::default);

    let config: UseLocalStorageHandle<VaultConfig> = use_local_storage("vault").unwrap();

    let on_clear_error = {
        let state = state.clone();
        Callback::from(move |()| {
            state.dispatch(AppAction::ClearError);
        })
    };

    let on_set_error = {
        let state = state.clone();
        Callback::from(move |msg| {
            state.dispatch(AppAction::JsOrSerdeError(msg));
        })
    };

    let on_init = {
        let config = config.clone();
        Callback::from(move |s: String| {
            let config = config.clone();
            spawn_local(async move {
                if let Err(err) = VaultConfig::new(s).await.map(|conf| config.set(Some(conf))) {
                    console::error_2(&"Init failed".into(), &err);
                }
            })
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

    let on_set_config = {
        let config = config.clone();
        Callback::from(move |new_config: Rc<VaultConfig>| {
            console::log_1(&"Updating config...".into());
            config.set_with_rc(Some(new_config))
        })
    };

    html! {
        <div class={classes!("wrapper")}>
            <div class={classes!("main-content")}>
                <div>
                    {
                        match &*config {
                            Some(Ok(config)) => {
                                html!{
                                    <Vault
                                        {config}
                                        set_config={on_set_config}
                                    />
                                }
                            }

                            None => {
                                html! {
                                    <>
                                        <p>{ "Vault is not initialized." }</p>
                                        <InitConfig on_submit={on_init} />
                                    </>
                                }
                            }

                            Some(Err(_)) => {
                                html! {
                                    <p>{ "Vault is corrupted." }</p>
                                }
                            }
                        }
                    }
                </div>

                <div class={classes!("flex-grow")} />

                <div>
                    <Collapse
                        button_text="Import vault config"
                        start_expanded={config.is_none()}
                        reverse_icon={true}
                    >
                        <Import on_import={on_import_config} ></Import>
                    </Collapse>
                </div>
            </div>

            <div class={classes!("footer")}>
                {"Footer"}
            </div>
        </div>
    }
}
