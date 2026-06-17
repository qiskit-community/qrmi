use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use pasqal_cloud_api::{AccessTokenRequest, Client, ClientBuilder};
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
    assert_eq!(Client::jwt_expiry_unix_seconds(&token).unwrap(), Some(exp));
}

#[test]
fn jwt_expiry_token_without_exp() {
    // This token does not have an "exp" claim, so the function should return None.
    let token = make_jwt(json!({"sub":"test"}));
    assert_eq!(Client::jwt_expiry_unix_seconds(&token).unwrap(), None);
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
    let past_token = make_jwt(json!({"exp": now - 600}));
    let future_token = make_jwt(json!({"exp": now + 600}));
    // Token that expires in 5 seconds should not be considered usable,
    // while token that expires in 15 seconds should be considered usable,
    // given the default buffer of 10 seconds.
    let near_expiry_token = make_jwt(json!({"exp": now + 5}));
    let over_buffer_token = make_jwt(json!({"exp": now + 15}));
    assert!(!Client::is_auth_token_usable(&past_token, now));
    assert!(!Client::is_auth_token_usable(&near_expiry_token, now));
    assert!(Client::is_auth_token_usable(&over_buffer_token, now));
    assert!(Client::is_auth_token_usable(&future_token, now));
}

#[test]
fn auth_token_usable_malformed_jwt() {
    let now = now_unix_seconds();
    assert!(!Client::is_auth_token_usable("a.@@@.c", now));
}

#[tokio::test]
async fn request_access_token_uses_username_password() {
    let mut server = mockito::Server::new_async().await;
    let token = make_jwt(json!({"sub":"user","exp":now_unix_seconds() + 3600}));
    let access_token_response = json!({
        "access_token": token,
        "expires_in": 3600,
        "token_type": "Bearer",
    });

    server
        .mock("POST", "/oauth/token")
        .with_status(200)
        .match_body(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded(
                "grant_type".to_string(),
                "http://auth0.com/oauth/grant-type/password-realm".to_string(),
            ),
            mockito::Matcher::UrlEncoded("username".to_string(), "mock-user".to_string()),
            mockito::Matcher::UrlEncoded("password".to_string(), "mock-password".to_string()),
        ]))
        .with_body(access_token_response.to_string())
        .create_async()
        .await;

    let auth_endpoint = format!("{}/oauth/token", server.url());
    let actual = Client::request_access_token(
        &auth_endpoint,
        AccessTokenRequest::UsernamePassword {
            username: "mock-user",
            password: "mock-password", // pragma: allowlist secret
        },
    )
    .await
    .expect("username/password token request should succeed");

    assert_eq!(actual, token);
}

#[tokio::test]
async fn request_access_token_uses_service_account_credentials() {
    let mut server = mockito::Server::new_async().await;
    let token = make_jwt(json!({"sub":"service-account","exp":now_unix_seconds() + 3600}));
    let access_token_response = json!({
        "access_token": token,
        "expires_in": 3600,
        "token_type": "Bearer",
    });

    server
        .mock("POST", "/oauth/token")
        .with_status(200)
        .match_body(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded(
                "grant_type".to_string(),
                "client_credentials".to_string(),
            ),
            mockito::Matcher::UrlEncoded("client_id".to_string(), "mock-client-id".to_string()),
            mockito::Matcher::UrlEncoded(
                "client_secret".to_string(),
                "mock-client-secret".to_string(),
            ),
        ]))
        .with_body(access_token_response.to_string())
        .create_async()
        .await;

    let auth_endpoint = format!("{}/oauth/token", server.url());
    let actual = Client::request_access_token(
        &auth_endpoint,
        AccessTokenRequest::ServiceAccount {
            client_id: "mock-client-id",
            client_secret: "mock-client-secret", // pragma: allowlist secret
        },
    )
    .await
    .expect("service account token request should succeed");

    assert_eq!(actual, token);
}

#[tokio::test]
async fn client_builder_service_account_credentials_refreshes_token_for_authenticated_request() {
    let mut server = mockito::Server::new_async().await;
    let token = make_jwt(json!({"sub":"service-account","exp":now_unix_seconds() + 3600}));
    let access_token_response = json!({
        "access_token": token,
        "expires_in": 3600,
        "token_type": "Bearer",
    });

    server
        .mock("POST", "/oauth/token")
        .with_status(200)
        .match_body(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded(
                "grant_type".to_string(),
                "client_credentials".to_string(),
            ),
            mockito::Matcher::UrlEncoded("client_id".to_string(), "mock-client-id".to_string()),
            mockito::Matcher::UrlEncoded(
                "client_secret".to_string(),
                "mock-client-secret".to_string(),
            ),
        ]))
        .with_body(access_token_response.to_string())
        .create_async()
        .await;

    server
        .mock("GET", "/core-fast/api/v2/batches/batch-id")
        .with_status(200)
        .match_header("authorization", format!("Bearer {token}").as_str())
        .with_body(json!({"data":{"status":"DONE","job_ids":["job-id"]}}).to_string())
        .create_async()
        .await;

    let mut builder = ClientBuilder::new("project-id".to_string());
    builder.with_auth_endpoint(format!("{}/oauth/token", server.url()));
    builder.with_base_url(server.url());
    builder.with_service_account_credentials(
        "mock-client-id".to_string(),
        "mock-client-secret".to_string(),
    );
    let mut client = builder.build().expect("client should build");

    let batch = client
        .get_batch("batch-id")
        .await
        .expect("authenticated request should succeed");

    assert_eq!(batch.data.job_ids, vec!["job-id".to_string()]);
}
