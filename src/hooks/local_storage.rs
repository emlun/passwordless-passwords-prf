use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::Deref;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::Storage;
use web_sys::StorageEvent;
use web_sys::Window;
use yew::hook;
use yew::use_mut_ref;
use yew::use_state;
use yew::UseStateHandle;

use crate::error::JsOrSerdeError;

#[derive(Debug)]
pub enum InitError {
    Unavailable,
    JsError(JsValue),
}

impl From<JsValue> for InitError {
    fn from(err: JsValue) -> Self {
        Self::JsError(err)
    }
}

type ParseResult<T> = Result<Rc<T>, (String, serde_json::Error)>;
type ListenerCallback = Closure<dyn FnMut(StorageEvent)>;

pub struct UseLocalStorageHandle<'name, T>
where
    T: Serialize,
    T: DeserializeOwned,
{
    storage: Rc<Storage>,
    name: &'name str,
    state: UseStateHandle<Option<ParseResult<T>>>,
}

impl<'name, T> Clone for UseLocalStorageHandle<'name, T>
where
    T: Serialize,
    T: DeserializeOwned,
{
    fn clone(&self) -> Self {
        Self {
            storage: Rc::clone(&self.storage),
            name: self.name,
            state: self.state.clone(),
        }
    }
}

impl<'name, T> Deref for UseLocalStorageHandle<'name, T>
where
    T: Serialize,
    T: DeserializeOwned,
{
    type Target = Option<ParseResult<T>>;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'name, T> UseLocalStorageHandle<'name, T>
where
    T: Serialize,
    T: DeserializeOwned,
{
    pub fn ok(&self) -> Option<Rc<T>> {
        self.as_ref()
            .and_then(|res| res.as_ref().ok().map(Rc::clone))
    }

    pub fn set(&self, value: Option<T>) -> Result<(), JsOrSerdeError> {
        self.set_with_rc(value.map(Rc::new))
    }

    pub fn set_with_rc(&self, value: Option<Rc<T>>) -> Result<(), JsOrSerdeError> {
        if let Some(value) = value {
            self.storage
                .set_item(self.name, &serde_json::to_string(&value)?)?;
            self.state.set(Some(Ok(value)));
        } else {
            self.storage.remove_item(self.name)?;
            self.state.set(None);
        }
        Ok(())
    }

    pub fn set_from_str(&self, value_str: &str) -> Result<(), JsOrSerdeError> {
        let value = serde_json::from_str(value_str)?;
        self.storage.set_item(self.name, value_str)?;
        self.state.set(Some(Ok(Rc::new(value))));
        Ok(())
    }

    fn deserialize(value: Option<String>) -> Option<ParseResult<T>> {
        value.map(|s| {
            serde_json::from_str(&s)
                .map(Rc::new)
                .map_err(|err| (s, err))
        })
    }

    fn on_storage_event(&self, event: StorageEvent) {
        if event.key().as_deref() == Some(self.name) {
            self.state.set(Self::deserialize(event.new_value()))
        }
    }
}

#[hook]
pub fn use_local_storage<T>(
    name: &'static str,
) -> Result<UseLocalStorageHandle<'static, T>, InitError>
where
    T: Serialize,
    T: DeserializeOwned,
    T: 'static,
{
    let window: Window = web_sys::window().ok_or(InitError::Unavailable)?;
    let storage: Storage = window.local_storage()?.ok_or(InitError::Unavailable)?;

    let value = storage.get_item(name)?;
    let state: UseStateHandle<Option<ParseResult<T>>> =
        { use_state(move || UseLocalStorageHandle::deserialize(value)) };

    let handle = UseLocalStorageHandle {
        storage: Rc::new(storage),
        name,
        state,
    };

    let listener: Rc<ListenerCallback> = {
        let handle = handle.clone();
        Rc::new(Closure::new(move |e: StorageEvent| {
            handle.on_storage_event(e);
        }))
    };

    let current_listener = {
        let listener = Rc::clone(&listener);
        use_mut_ref(move || listener)
    };

    if let Err(err) = window.remove_event_listener_with_callback(
        "storage",
        Closure::as_ref(Rc::as_ref(&current_listener.borrow())).unchecked_ref(),
    ) {
        console::error_2(
            &format!("Failed to update storage event listener for \"{name}\"").into(),
            &err,
        );
    } else {
        window.add_event_listener_with_callback(
            "storage",
            Closure::as_ref(Rc::as_ref(&listener)).unchecked_ref(),
        )?;
    }
    *current_listener.borrow_mut() = listener;

    Ok(handle)
}
