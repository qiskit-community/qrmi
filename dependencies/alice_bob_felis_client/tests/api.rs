use anyhow::Result;
use dotenvy::dotenv;
use felis::apis::{configuration, health_service, targets_service};
use felis::helpers::decode_api_key;

async fn prepare_config() -> Result<configuration::Configuration, anyhow::Error> {
    dotenv().ok();
    let api_key = std::env::var("FELIS_API_KEY")?;
    let endpoint = std::env::var("FELIS_BASE_ENDPOINT")?;

    let mut config = configuration::Configuration::new();
    config.base_path = endpoint;
    config.basic_auth = decode_api_key(&api_key).unwrap();

    Ok(config)
}

#[tokio::test]
async fn test_health_check() -> Result<(), anyhow::Error> {
    let config = prepare_config().await?;
    let response = health_service::check_health(&config, None).await?;
    assert_eq!(response, "OK");
    Ok(())
}

#[tokio::test]
async fn test_list_targets() -> Result<(), anyhow::Error> {
    let config = prepare_config().await?;
    targets_service::list_targets(&config).await?;
    Ok(())
}
