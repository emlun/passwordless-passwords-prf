use ::base64::Engine;
use js_sys::Uint8Array;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::JsValue;
use web_sys::PublicKeyCredentialDescriptor;
use web_sys::PublicKeyCredentialType;

use crate::crypto::gen_random;

pub mod vault;

pub mod base64 {
    use ::base64::Engine;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;

    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct Base64Wrapper {
        #[serde(rename = "$base64")]
        base64: String,
    }

    impl From<&Vec<u8>> for Base64Wrapper {
        fn from(v: &Vec<u8>) -> Self {
            Self {
                base64: ::base64::engine::general_purpose::STANDARD.encode(v),
            }
        }
    }

    impl TryFrom<Base64Wrapper> for Vec<u8> {
        type Error = base64::DecodeError;
        fn try_from(v: Base64Wrapper) -> Result<Self, Self::Error> {
            Ok(::base64::engine::general_purpose::STANDARD.decode(v.base64)?)
        }
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: TryFrom<Base64Wrapper>,
        <T as TryFrom<Base64Wrapper>>::Error: std::fmt::Display,
    {
        let b64: Base64Wrapper = Deserialize::deserialize(d)?;
        T::try_from(b64).map_err(|e| serde::de::Error::custom(e))
    }

    pub fn serialize<'t, S, T>(v: &'t T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        Base64Wrapper: From<&'t T>,
    {
        let b64: Base64Wrapper = Base64Wrapper::from(v);
        b64.serialize(s)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct UserHandle(#[serde(with = "crate::data::base64")] Vec<u8>);

impl UserHandle {
    pub async fn generate() -> Result<Self, JsValue> {
        Ok(Self(Vec::from(gen_random::<64>()?)))
    }

    pub fn uint8_array(&self) -> Uint8Array {
        let Self(v) = self;
        Uint8Array::from(v.as_slice())
    }
}

#[derive(Clone, PartialEq)]
pub struct CredentialId(Vec<u8>);

impl From<Vec<u8>> for CredentialId {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl CredentialId {
    pub fn b64_abbrev(&self, max_len: usize) -> String {
        let Self(v) = self;
        let b64 = ::base64::engine::general_purpose::STANDARD.encode(v);

        if b64.len() <= max_len {
            b64.clone()
        } else {
            format!("{}…", &b64[0..max_len])
        }
    }

    pub fn b64url(&self) -> String {
        let Self(v) = self;
        ::base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(v)
    }
}

impl From<&CredentialId> for PublicKeyCredentialDescriptor {
    fn from(v: &CredentialId) -> PublicKeyCredentialDescriptor {
        PublicKeyCredentialDescriptor::new(&Uint8Array::from(v), PublicKeyCredentialType::PublicKey)
    }
}

impl From<&CredentialId> for Uint8Array {
    fn from(v: &CredentialId) -> Uint8Array {
        Uint8Array::from(v.0.as_slice())
    }
}

// impl From<&CredentialId> for yew::virtual_dom::Key {
//     fn from(val: &CredentialId) -> Self {
//         (val.b64.0.as_str()).into()
//     }
// }

impl From<CredentialId> for yew::virtual_dom::Key {
    fn from(val: CredentialId) -> Self {
        let CredentialId(v) = val;
        let b64 = ::base64::engine::general_purpose::STANDARD.encode(v);
        b64.into()
    }
}
