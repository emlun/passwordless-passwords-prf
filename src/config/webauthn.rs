use web_sys::PublicKeyCredentialRpEntity;

#[cfg(debug_assertions)]
pub fn rp_id() -> &'static str {
    option_env!("RP_ID").unwrap_or("localhost")
}

#[cfg(not(debug_assertions))]
pub fn rp_id() -> &'static str {
    env!("RP_ID", "Please set the WebAuthn RP ID for where you are planning to deploy the application; see: https://www.w3.org/TR/2021/REC-webauthn-2-20210408/#rp-id")
}

#[cfg(debug_assertions)]
pub fn rp_name() -> &'static str {
    option_env!("RP_NAME").unwrap_or(env!("CARGO_PKG_NAME"))
}

#[cfg(not(debug_assertions))]
pub fn rp_name() -> &'static str {
    env!("RP_NAME", "Please set the WebAuthn RP name; see: https://www.w3.org/TR/2021/REC-webauthn-2-20210408/#dom-publickeycredentialentity-name")
}

pub fn rp_entity() -> PublicKeyCredentialRpEntity {
    PublicKeyCredentialRpEntity::new(rp_name())
        .id(rp_id())
        .to_owned()
}
