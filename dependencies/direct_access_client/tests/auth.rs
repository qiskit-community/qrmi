//
// (C) Copyright IBM 2024, 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

mod common;
use assert_json_diff::assert_json_include;
#[cfg(feature = "ibmcloud_appid_auth")]
use base64::{engine::general_purpose::STANDARD, prelude::*};
use direct_access_api::{AuthMethod, ClientBuilder};
use serde_json::json;

use chrono::Utc;
use hmac::{Hmac, Mac};
use jwt::{RegisteredClaims, SignWithKey};
use sha2::Sha256;

/// Generates new JWT token for access-token authentication
fn new_token() -> Result<String, &'static str> {
    let now = Utc::now().timestamp();
    let exp = now + 3600;
    let claims = RegisteredClaims {
        subject: Some("test_subject".to_string()),
        issued_at: Some(now.try_into().unwrap()),
        expiration: Some(exp.try_into().unwrap()),
        ..Default::default()
    };

    let key: Hmac<Sha256> = Hmac::new_from_slice(b"secret_key").map_err(|_e| "Invalid key")?;

    let signed_token = claims.sign_with_key(&key).map_err(|_e| "Sign failed")?;

    Ok(signed_token)
}

/// Test with valid access token.
/// Client.list_backends() should return the backend list as expected.
#[tokio::test]
#[cfg(feature = "ibmcloud_appid_auth")]
async fn test_auth_access_token() {
    common::setup();

    const APPID_USERID: &str = "demo";
    const APPID_PASSWORD: &str = "demopass";

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "backends": [
            {
                "name": "backend_online",
                "status": "online",
            },
            {
                "name": "backend_offline",
                "status": "offline",
            },
            {
                "name": "backend_paused",
                "status": "paused",
            },
        ],
    });

    let token = new_token().unwrap();
    let access_token_response = json!({
        "access_token": token,
        "expires_in": 3600,
        "token_type": "Bearer",
    });

    let base64_str = STANDARD.encode(format!("{}:{}", APPID_USERID, APPID_PASSWORD).as_bytes());
    server
        .mock("POST", "/v1/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .match_header("authorization", format!("Basic {}", base64_str).as_str())
        .with_body(access_token_response.to_string())
        .create_async()
        .await;

    server
        .mock("GET", "/v1/backends")
        .with_status(200)
        .with_header("content-type", "application/json")
        .match_header("authorization", format!("Bearer {}", token).as_str())
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url)
        .with_auth(AuthMethod::IbmCloudAppId {
            username: APPID_USERID.to_string(),
            password: APPID_PASSWORD.to_string(),
        })
        .build()
        .unwrap();
    let actual = client.list_backends::<serde_json::Value>().await.unwrap();
    assert_json_include!(actual: actual, expected: expected);
}

/// Test with invalid AppID credential.
/// Client.list_backends() should be failed with error message.
#[tokio::test]
#[cfg(feature = "ibmcloud_appid_auth")]
async fn test_auth_invalid_appid() {
    common::setup();

    const APPID_USERID: &str = "demo";
    const APPID_PASSWORD: &str = "demopass";

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "status_code": 400,
        "title": "Error authenticating user.",
        "trace": "",
        "errors": [
            {
                "code": "1219",
                "message": "Error authenticating user.",
                "more_info": "https://cloud.ibm.com/apidocs/quantum-computing#error-handling"
            }
        ]
    });

    let base64_str = STANDARD.encode(format!("{}:{}", APPID_USERID, APPID_PASSWORD).as_bytes());
    server
        .mock("POST", "/v1/token")
        .with_status(400)
        .with_header("content-type", "application/json")
        .match_header("authorization", format!("Basic {}", base64_str).as_str())
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url)
        .with_auth(AuthMethod::IbmCloudAppId {
            username: APPID_USERID.to_string(),
            password: APPID_PASSWORD.to_string(),
        })
        .build()
        .unwrap();
    let actual = client.list_backends::<serde_json::Value>().await;
    assert!(actual.is_err());

    if let Err(e) = actual {
        println!("Error message return by API: {}", e)
    }
}

/// Test with invalid access token.
/// Client.list_backends() should be failed with error message.
#[tokio::test]
#[cfg(feature = "ibmcloud_appid_auth")]
async fn test_auth_invalid_access_token() {
    common::setup();

    const APPID_USERID: &str = "demo";
    const APPID_PASSWORD: &str = "demopass";

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "status_code": 401,
        "title": "Invalid credentials.",
        "trace": "",
        "errors": [
            {
                "code": "1201",
                "message": "Invalid credentials.",
                "more_info": "https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
            }
        ],
    });

    let token = new_token().unwrap();
    let access_token_response = json!({
        "access_token": token,
        "expires_in": 3600,
        "token_type": "Bearer",
    });

    let base64_str = STANDARD.encode(format!("{}:{}", APPID_USERID, APPID_PASSWORD).as_bytes());
    server
        .mock("POST", "/v1/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .match_header("authorization", format!("Basic {}", base64_str).as_str())
        .with_body(access_token_response.to_string())
        .create_async()
        .await;

    server
        .mock("GET", "/v1/backends")
        .with_status(401)
        .with_header("content-type", "application/json")
        .match_header("authorization", format!("Bearer {}", token).as_str())
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url)
        .with_auth(AuthMethod::IbmCloudAppId {
            username: APPID_USERID.to_string(),
            password: APPID_PASSWORD.to_string(),
        })
        .build()
        .unwrap();
    let actual = client.list_backends::<serde_json::Value>().await;
    assert!(actual.is_err());

    if let Err(e) = actual {
        println!("Error message return by API: {}", e)
    }
}

/// Test with valid internal shared key.
/// Client.list_backends() should return the backend list as expected.
#[tokio::test]
#[cfg(feature = "internal_shared_key_auth")]
async fn test_internal_shared_key() {
    common::setup();

    const CLIENT_ID: &str = "demo";
    const SHARED_TOKEN: &str = "demopass";

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "backends": [
            {
                "name": "backend_online",
                "status": "online",
            },
            {
                "name": "backend_offline",
                "status": "offline",
            },
            {
                "name": "backend_paused",
                "status": "paused",
            },
        ],
    });

    server
        .mock("GET", "/v1/backends")
        .with_status(200)
        .with_header("content-type", "application/json")
        .match_header(
            "authorization",
            format!("apikey {}:{}", CLIENT_ID, SHARED_TOKEN).as_str(),
        )
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url)
        .with_auth(AuthMethod::InternalSharedKey {
            client_id: CLIENT_ID.to_string(),
            shared_token: SHARED_TOKEN.to_string(),
        })
        .build()
        .unwrap();
    let actual = client.list_backends::<serde_json::Value>().await.unwrap();
    assert_json_include!(actual: actual, expected: expected);
}

/// Test with invalid internal shared key.
/// Client.list_backends() should be failed with error message.
#[tokio::test]
#[cfg(feature = "internal_shared_key_auth")]
async fn test_invalid_internal_shared_key() {
    common::setup();

    const CLIENT_ID: &str = "demo";
    const SHARED_TOKEN: &str = "demopass";
    const INVALID_SHARED_TOKEN: &str = "invalidpass";

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "status_code": 401,
        "title": "Invalid credentials.",
        "trace": "",
        "errors": [
            {
                "code": "1201",
                "message": "Invalid credentials.",
                "more_info": "https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
            }
        ],
    });

    server
        .mock("GET", "/v1/backends")
        .with_status(401)
        .with_header("content-type", "application/json")
        .match_header(
            "authorization",
            format!("apikey {}:{}", CLIENT_ID, SHARED_TOKEN).as_str(),
        )
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url)
        .with_auth(AuthMethod::InternalSharedKey {
            client_id: CLIENT_ID.to_string(),
            shared_token: INVALID_SHARED_TOKEN.to_string(),
        })
        .build()
        .unwrap();
    let actual = client.list_backends::<serde_json::Value>().await;
    assert!(actual.is_err());

    if let Err(e) = actual {
        println!("Error message return by API: {}", e)
    }
}

/// Test with valid IAM bearer token.
/// Client.list_backends() should return the backend list as expected.
#[tokio::test]
async fn test_auth_iam_bearer_token() {
    common::setup();

    const IAM_APIKEY: &str = "demoapikey";
    const SERVICE_CRN: &str = "crn:v1:local:test";
    const GRANT_TYPE: &str = "urn:ibm:params:oauth:grant-type:apikey";

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "backends": [
            {
                "name": "backend_online",
                "status": "online",
            },
            {
                "name": "backend_offline",
                "status": "offline",
            },
            {
                "name": "backend_paused",
                "status": "paused",
            },
        ],
    });

    let token = new_token().unwrap();
    let access_token_response = json!({
        "access_token": token,
        "expires_in": 3600,
        "token_type": "Bearer",
    });

    server
        .mock("POST", "/identity/token")
        .with_status(200)
        .with_header("content-type", "application/x-www-form-urlencoded")
        .match_body(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("grant_type".to_string(), GRANT_TYPE.to_string()),
            mockito::Matcher::UrlEncoded("apikey".to_string(), IAM_APIKEY.to_string()),
        ]))
        .with_body(access_token_response.to_string())
        .create_async()
        .await;

    server
        .mock("GET", "/v1/backends")
        .with_status(200)
        .with_header("content-type", "application/json")
        .match_header("authorization", format!("Bearer {}", token).as_str())
        .match_header("service-crn", SERVICE_CRN)
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url)
        .with_auth(AuthMethod::IbmCloudIam {
            apikey: IAM_APIKEY.to_string(),
            service_crn: SERVICE_CRN.to_string(),
            iam_endpoint_url: base_url,
        })
        .build()
        .unwrap();
    let actual = client.list_backends::<serde_json::Value>().await.unwrap();
    assert_json_include!(actual: actual, expected: expected);
}

/// Test with invalid IAM API key.
/// Client.list_backends() should be failed with error message.
#[tokio::test]
async fn test_auth_invalid_iam_apikey() {
    common::setup();

    const IAM_APIKEY: &str = "demoapikey";
    const SERVICE_CRN: &str = "crn:v1:local:test";

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "status_code": 400,
        "title": "Error authenticating user.",
        "trace": "",
        "errors": [
            {
                "code": "1219",
                "message": "Error authenticating user.",
                "more_info": "https://cloud.ibm.com/apidocs/quantum-computing#error-handling"
            }
        ]
    });

    server
        .mock("POST", "/identity/token")
        .with_status(400)
        .with_header("content-type", "application/x-www-form-urlencoded")
        .match_body(
            format!(
                "grant_type=urn:ibm:params:oauth:grant-type:apikey&apikey={}",
                IAM_APIKEY
            )
            .as_str(),
        )
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url)
        .with_auth(AuthMethod::IbmCloudIam {
            apikey: IAM_APIKEY.to_string(),
            service_crn: SERVICE_CRN.to_string(),
            iam_endpoint_url: base_url,
        })
        .build()
        .unwrap();
    let actual = client.list_backends::<serde_json::Value>().await;
    assert!(actual.is_err());

    if let Err(e) = actual {
        println!("Error message return by API: {}", e)
    }
}

/// Test with invalid IAM bearer token.
/// Client.list_backends() should be failed with error message.
#[tokio::test]
async fn test_auth_invalid_iam_bearer_token() {
    common::setup();

    const IAM_APIKEY: &str = "demoapikey";
    const SERVICE_CRN: &str = "crn:v1:local:test";

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "status_code": 401,
        "title": "Invalid credentials.",
        "trace": "",
        "errors": [
            {
                "code": "1201",
                "message": "Invalid credentials.",
                "more_info": "https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
            }
        ],
    });

    let token = new_token().unwrap();
    let access_token_response = json!({
        "access_token": token,
        "expires_in": 3600,
        "token_type": "Bearer",
    });

    server
        .mock("POST", "/identity/token")
        .with_status(200)
        .with_header("content-type", "application/x-www-form-urlencoded")
        .match_body(
            format!(
                "grant_type=urn:ibm:params:oauth:grant-type:apikey&apikey={}",
                IAM_APIKEY
            )
            .as_str(),
        )
        .with_body(access_token_response.to_string())
        .create_async()
        .await;

    server
        .mock("GET", "/v1/backends")
        .with_status(401)
        .with_header("content-type", "application/json")
        .match_header("authorization", format!("Bearer {}", token).as_str())
        .match_header("service-crn", SERVICE_CRN)
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url)
        .with_auth(AuthMethod::IbmCloudIam {
            apikey: IAM_APIKEY.to_string(),
            service_crn: SERVICE_CRN.to_string(),
            iam_endpoint_url: base_url,
        })
        .build()
        .unwrap();
    let actual = client.list_backends::<serde_json::Value>().await;
    assert!(actual.is_err());

    if let Err(e) = actual {
        println!("Error message return by API: {}", e)
    }
}
