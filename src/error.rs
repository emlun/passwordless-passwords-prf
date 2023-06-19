use js_sys::Object;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum JsOrSerdeError {
    JsError(JsValue),
    SerializeError(serde_json::Error),
}

impl std::error::Error for JsOrSerdeError {}

impl std::fmt::Display for JsOrSerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::JsError(js_value) => write!(f, "JavaScript error: {js_value:?}",),
            Self::SerializeError(err) => write!(f, "Serialization failed: {err}",),
        }
    }
}

impl From<JsValue> for JsOrSerdeError {
    fn from(err: JsValue) -> Self {
        Self::JsError(err)
    }
}

impl From<Object> for JsOrSerdeError {
    fn from(err: Object) -> Self {
        Self::from(JsValue::from(err))
    }
}

impl From<serde_json::Error> for JsOrSerdeError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializeError(err)
    }
}
