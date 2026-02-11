use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use pasqal_cloud_api::Client;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be after UNIX_EPOCH")
        .as_secs() as i64
}

fn make_jwt(payload: serde_json::Value) -> String {
    let header = json!({"alg":"none","typ":"JWT"});
    let header = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
    let payload = URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
    format!("{header}.{payload}.sig")
}

#[test]
fn jwt_expiry_valid_token_with_exp() {
    // This token has a valid "exp" claim, so the function should return it.
    let exp = now_unix_seconds() + 3600;
    let token = make_jwt(json!({"sub":"test","exp":exp}));
    assert_eq!(
        Client::jwt_expiry_unix_seconds(&token).unwrap(),
        Some(exp)
    );
}

#[test]
fn jwt_expiry_token_without_exp() {
    // This token does not have an "exp" claim, so the function should return None.
    let token = make_jwt(json!({"sub":"test"}));
    assert_eq!(
        Client::jwt_expiry_unix_seconds(&token).unwrap(),
        None
    );
}

#[test]
fn jwt_expiry_non_jwt_token() {
    // This is not a JWT, but the function should handle it gracefully and return None.
    assert_eq!(
        Client::jwt_expiry_unix_seconds("opaque_token").unwrap(),
        None
    );
}

#[test]
fn jwt_expiry_malformed_token() {
    // This token is JWT but malformed so the function errors.
    let err = Client::jwt_expiry_unix_seconds("a.@@@.c");
    assert!(err.is_err());
}

#[test]
fn jwt_expiry_past_and_future() {
    // We create two tokens:
    // one with an "exp" in the past and one with an "exp" in the future,
    // verify that the function returns the correct values for both.
    let now = now_unix_seconds();
    let past = now - 60;
    let future = now + 60;
    let past_token = make_jwt(json!({"exp":past}));
    let future_token = make_jwt(json!({"exp":future}));
    assert_eq!(
        Client::jwt_expiry_unix_seconds(&past_token).unwrap(),
        Some(past)
    );
    assert_eq!(
        Client::jwt_expiry_unix_seconds(&future_token).unwrap(),
        Some(future)
    );
}

#[test]
fn auth_token_usable_empty_token() {
    let now = now_unix_seconds();
    assert!(!Client::is_auth_token_usable("", now));
}

#[test]
fn auth_token_usable_without_expiry() {
    let now = now_unix_seconds();
    assert!(Client::is_auth_token_usable("opaque_token", now));
}

#[test]
fn auth_token_usable_with_expiry() {
    let now = now_unix_seconds();
    let past_token = make_jwt(json!({"exp": now - 60}));
    let future_token = make_jwt(json!({"exp": now + 60}));
    assert!(Client::is_auth_token_usable(&future_token, now));
    assert!(!Client::is_auth_token_usable(&past_token, now));
}

#[test]
fn auth_token_usable_malformed_jwt() {
    let now = now_unix_seconds();
    assert!(!Client::is_auth_token_usable("a.@@@.c", now));
}
