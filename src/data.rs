use js_sys::ArrayBuffer;
use web_sys::PublicKeyCredential;

#[derive(Clone, PartialEq)]
pub struct Base64Url(pub String);

#[derive(Clone, PartialEq)]
pub struct CredentialId {
    pub raw: ArrayBuffer,
    pub b64: Base64Url,
}

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
