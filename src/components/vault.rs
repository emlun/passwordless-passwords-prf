use std::rc::Rc;
use stylist::yew::styled_component;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::html;
use yew::Callback;
use yew::Html;
use yew::Properties;

use crate::components::create_button::CreateButton;
use crate::components::credentials_list::CredentialsList;
use crate::components::files_list::FilesList;
use crate::components::insert_content::InsertContent;
use crate::crypto::WrappedKeypair;
use crate::data::vault::VaultConfig;
use crate::data::CredentialId;
use crate::error::JsOrSerdeError;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub config: Rc<VaultConfig>,
    pub set_config: Callback<Rc<VaultConfig>, Result<(), JsOrSerdeError>>,
}

#[styled_component]
pub fn Vault(props: &Props) -> Html {
    let on_create = {
        let set_config = props.set_config.clone();
        let conf: Rc<VaultConfig> = Rc::clone(&props.config);

        Callback::from(move |wrapped_keypair: WrappedKeypair| {
            let mut conf = Rc::clone(&conf);
            Rc::make_mut(&mut Rc::make_mut(&mut Rc::make_mut(&mut conf).user).keypairs)
                .push(Rc::new(wrapped_keypair));
            match set_config.emit(conf) {
                Ok(()) => {
                    console::log_1(&"Successfully registered new credential!".into());
                }
                Err(JsOrSerdeError::JsError(e)) => {
                    console::log_2(&"Failed to register new credential:".into(), &e);
                }
                Err(JsOrSerdeError::SerializeError(_)) => {
                    console::log_1(
                        &"Failed to register new credential: JSON serialization failed.".into(),
                    );
                }
            }
        })
    };

    let on_delete = {
        let set_config = props.set_config.clone();
        let conf: Rc<VaultConfig> = Rc::clone(&props.config);

        Callback::from(move |cred_id: CredentialId| {
            let mut conf = Rc::clone(&conf);
            Rc::make_mut(&mut conf).delete_credential(&cred_id);
            match set_config.emit(conf) {
                Ok(()) => {
                    console::log_1(&"Successfully deleted credential!".into());
                }
                Err(JsOrSerdeError::JsError(e)) => {
                    console::log_2(&"Failed to delete credential:".into(), &e);
                }
                Err(JsOrSerdeError::SerializeError(_)) => {
                    console::log_1(
                        &"Failed to delete credential: JSON serialization failed.".into(),
                    );
                }
            }
        })
    };

    let on_rename_credential = {
        let set_config = props.set_config.clone();
        let conf: Rc<VaultConfig> = Rc::clone(&props.config);

        Callback::from(move |(cred_id, name): (CredentialId, String)| {
            let mut conf = Rc::clone(&conf);
            let result = Rc::make_mut(&mut conf).rename_credential(&cred_id, name);

            match result
                .map(|_|
                     // Throw away the returned reference to conf, but preserve the error if any
                     ())
                .and_then(|_| set_config.emit(conf))
            {
                Ok(()) => {
                    console::log_1(&"Successfully renamed credential!".into());
                }
                Err(JsOrSerdeError::JsError(e)) => {
                    console::log_2(&"Failed to rename credential:".into(), &e);
                }
                Err(JsOrSerdeError::SerializeError(_)) => {
                    console::log_1(
                        &"Failed to rename credential: JSON serialization failed.".into(),
                    );
                }
            }
        })
    };

    let on_insert = {
        let set_config = props.set_config.clone();
        let conf: Rc<VaultConfig> = Rc::clone(&props.config);

        Callback::from(move |(name, content): (String, Vec<u8>)| {
            let set_config = set_config.clone();
            let conf = Rc::clone(&conf);
            spawn_local(async move {
                let mut conf = Rc::clone(&conf);
                let result = Rc::make_mut(&mut conf).push_content(name, content).await;
                match result
                    .map(|_|
                                 // Throw away the returned reference to conf, but preserve the error if any
                                 ())
                    .and_then(|_| set_config.emit(conf))
                {
                    Ok(()) => {
                        console::log_1(&"Successfully encrypted content!".into());
                    }
                    Err(JsOrSerdeError::JsError(e)) => {
                        console::log_2(&"Failed to encrypt content:".into(), &e);
                    }
                    Err(JsOrSerdeError::SerializeError(_)) => {
                        console::log_1(
                            &"Failed to encrypt content: JSON serialization failed.".into(),
                        );
                    }
                }
            })
        })
    };

    html! {
        <>
            <div>
                <CreateButton
                    config={Rc::clone(&props.config)}
                    {on_create}
                    on_begin={|_| {}}
                    on_fail={|_| {}}
                />
            </div>
            <div>
                <CredentialsList
                    keypairs={Rc::clone(&props.config.user.keypairs)}
                    {on_delete}
                    on_rename={on_rename_credential}
                />
            </div>
            <div>
                <FilesList config={Rc::clone(&props.config)} />
            </div>
            <div>
                <InsertContent
                    config={Rc::clone(&props.config)}
                    on_submit={on_insert}
                />
            </div>
        </>
    }
}
