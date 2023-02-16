use base64::Engine;
use js_sys::ArrayBuffer;
use js_sys::Uint8Array;
use serde::Deserialize;
use serde::Serialize;
use web_sys::PublicKeyCredential;

pub mod vault;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(try_from = "&str", into = "String")]
pub struct Base64(pub Vec<u8>);

impl TryFrom<&str> for Base64 {
    type Error = base64::DecodeError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self(base64::engine::general_purpose::STANDARD.decode(s)?))
    }
}

impl TryFrom<ArrayBuffer> for Base64 {
    type Error = <Self as TryFrom<Uint8Array>>::Error;
    fn try_from(arr: ArrayBuffer) -> Result<Self, Self::Error> {
        Self::try_from(Uint8Array::new(&arr))
    }
}

impl From<Base64> for String {
    fn from(Base64(v): Base64) -> Self {
        base64::engine::general_purpose::STANDARD.encode(v)
    }
}

impl TryFrom<&Base64> for Uint8Array {
    type Error = std::num::TryFromIntError;
    fn try_from(Base64(v): &Base64) -> Result<Self, Self::Error> {
        let u = Uint8Array::new_with_length(v.len().try_into()?);
        u.copy_from(v);
        Ok(u)
    }
}

impl TryFrom<Uint8Array> for Base64 {
    type Error = std::num::TryFromIntError;
    fn try_from(arr: Uint8Array) -> Result<Self, Self::Error> {
        let mut result = Base64(vec![0; usize::try_from(arr.byte_length())?]);
        arr.copy_to(&mut result.0);
        Ok(result)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(from = "Base64", into = "Base64")]
pub struct UserHandle(Vec<u8>);

impl From<UserHandle> for Base64 {
    fn from(UserHandle(v): UserHandle) -> Self {
        Self(v)
    }
}

impl From<Base64> for UserHandle {
    fn from(Base64(v): Base64) -> Self {
        Self(v)
    }
}

#[derive(Clone, PartialEq)]
pub struct Base64Url(pub String);

#[derive(Clone, PartialEq)]
// #[derive(Clone, Deserialize, PartialEq, Serialize)]
// #[serde(from = "Base64", into = "Base64")]
pub struct CredentialId {
    pub raw: ArrayBuffer,
    pub b64: Base64Url,
}

// impl From<CredentialId> for Base64 {
//     fn from(cred_id: CredentialId) -> Self {
//         Self(Uint8Array::new(&cred_id.raw).to_vec())
//     }
// }

// impl From<Base64> for CredentialId {
//     fn from(Base64(v): Base64) -> Self {
//         Self {
//             b64: Base64Url(base64::engine::general_purpose::STANDARD.encode(&v)),
//             raw: Uint8Array::from(&v).buffer(),
//         }
//     }
// }

impl CredentialId {
    pub fn b64_abbrev(&self, max_len: usize) -> String {
        if self.b64.0.len() <= max_len {
            self.b64.0.clone()
        } else {
            format!("{}…", &self.b64.0[0..max_len])
        }
    }
}

impl From<&CredentialId> for yew::virtual_dom::Key {
    fn from(val: &CredentialId) -> Self {
        (val.b64.0.as_str()).into()
    }
}

impl From<CredentialId> for yew::virtual_dom::Key {
    fn from(val: CredentialId) -> Self {
        (val.b64.0.as_str()).into()
    }
}

#[derive(Clone, PartialEq)]
pub struct Credential {
    pub id: CredentialId,
    pub nickname: Option<String>,
}

impl From<PublicKeyCredential> for Credential {
    fn from(cred: PublicKeyCredential) -> Self {
        Self {
            id: CredentialId {
                raw: cred.raw_id(),
                b64: Base64Url(cred.id()),
            },
            nickname: None,
        }
    }
}
