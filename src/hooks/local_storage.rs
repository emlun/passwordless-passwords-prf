use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::Deref;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::Storage;
use yew::hook;
use yew::use_state;
use yew::UseStateHandle;

#[derive(Debug)]
pub enum InitError {
    Unavailable,
    JsError(JsValue),
}

#[derive(Debug)]
pub enum SetError {
    JsError(JsValue),
    SerializeError(serde_json::Error),
}

impl From<JsValue> for InitError {
    fn from(err: JsValue) -> Self {
        Self::JsError(err)
    }
}

impl From<JsValue> for SetError {
    fn from(err: JsValue) -> Self {
        Self::JsError(err)
    }
}

impl From<serde_json::Error> for SetError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializeError(err)
    }
}

type ParseResult<T> = Result<Rc<T>, (String, serde_json::Error)>;

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

    pub fn set(&self, value: Option<T>) -> Result<(), SetError> {
        if let Some(value) = value {
            self.storage
                .set_item(self.name, &serde_json::to_string(&value)?)?;
            self.state.set(Some(Ok(Rc::new(value))));
        } else {
            self.storage.remove_item(self.name)?;
            self.state.set(None);
        }
        Ok(())
    }

    pub fn set_from_str(&self, value_str: &str) -> Result<(), SetError> {
        let value = serde_json::from_str(value_str)?;
        self.storage.set_item(self.name, value_str)?;
        self.state.set(Some(Ok(Rc::new(value))));
        Ok(())
    }
}

#[hook]
pub fn use_local_storage<'name, T>(
    name: &'name str,
) -> Result<UseLocalStorageHandle<'name, T>, InitError>
where
    T: Serialize,
    T: DeserializeOwned,
    T: 'static,
{
    let storage: Storage = web_sys::window()
        .ok_or(InitError::Unavailable)?
        .local_storage()?
        .ok_or(InitError::Unavailable)?;

    let value = storage.get_item(name)?;

    let state: UseStateHandle<Option<ParseResult<T>>> = use_state(move || {
        value.map(|s| {
            serde_json::from_str(&s)
                .map(Rc::new)
                .map_err(|err| (s, err))
        })
    });

    Ok(UseLocalStorageHandle {
        storage: Rc::new(storage),
        name,
        state,
    })
}
