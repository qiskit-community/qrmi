use alice_bob_felis::apis::{configuration, health_service, targets_service};
use alice_bob_felis::helpers::decode_api_key;

const API_KEY: &str = "dXNlcjpwYXNzd29yZA=="; // pragma: allowlist secret

fn prepare_config(endpoint: String) -> configuration::Configuration {
    let mut config = configuration::Configuration::new();
    config.base_path = endpoint;
    config.basic_auth = decode_api_key(API_KEY).unwrap();

    config
}

#[tokio::test]
async fn test_health_check() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/health/")
        .match_header("authorization", format!("Basic {API_KEY}").as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("\"OK\"")
        .create_async()
        .await;

    let config = prepare_config(server.url());
    let response = health_service::check_health(&config, None).await.unwrap();

    assert_eq!(response, "OK");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_targets() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/targets/")
        .match_header("authorization", format!("Basic {API_KEY}").as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[]")
        .create_async()
        .await;

    let config = prepare_config(server.url());
    let response = targets_service::list_targets(&config).await.unwrap();

    assert!(response.is_empty());
    mock.assert_async().await;
}
