use serde::de::DeserializeOwned;
use wasm_bindgen::JsValue;
use web_sys::Storage;
use yew::hook;

#[derive(Debug)]
pub enum Error {
    Unavailable,
    JsError(JsValue),
    DeserializationError(serde_json::Error),
}

impl From<JsValue> for Error {
    fn from(err: JsValue) -> Self {
        Self::JsError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::DeserializationError(err)
    }
}

#[hook]
pub fn use_local_storage<T>(name: &str) -> Result<Option<T>, Error>
where
    T: DeserializeOwned,
{
    let local_storage: Storage = web_sys::window()
        .ok_or(Error::Unavailable)?
        .local_storage()?
        .ok_or(Error::Unavailable)?;

    if let Some(s) = local_storage.get_item(name)? {
        Ok(serde_json::from_str(&s).map(|ok| Some(ok))?)
    } else {
        Ok(None)
    }
}
