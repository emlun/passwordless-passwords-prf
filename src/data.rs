use js_sys::ArrayBuffer;
use std::fmt::Display;
use web_sys::PublicKeyCredential;

#[derive(Clone, PartialEq)]
pub struct Base64Url(String);

impl Display for Base64Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

#[derive(Clone, PartialEq)]
pub struct CredentialId {
    pub raw: ArrayBuffer,
    pub b64: Base64Url,
}

impl From<&CredentialId> for yew::virtual_dom::Key {
    fn from(val: &CredentialId) -> Self {
        (val.b64.0.as_str()).into()
    }
}

#[derive(Clone, PartialEq)]
pub struct Credential {
    pub id: CredentialId,
}

impl From<PublicKeyCredential> for Credential {
    fn from(cred: PublicKeyCredential) -> Self {
        Self {
            id: CredentialId {
                raw: cred.raw_id(),
                b64: Base64Url(cred.id()),
            },
        }
    }
}
