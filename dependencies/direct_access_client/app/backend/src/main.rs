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

use retry_policies::policies::ExponentialBackoff;
use retry_policies::Jitter;
use std::time::Duration;

use direct_access_api::models::{Backend, Backends};
use direct_access_api::{AuthMethod, ClientBuilder};

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
        .jitter(Jitter::Full)
        .base(2)
        .build_with_max_retries(5);

    let client = ClientBuilder::new("http://0.0.0.0:8290")
        .with_auth(AuthMethod::IbmCloudIam {
            apikey: "demoapikey1".to_string(),
            service_crn: "crn:v1:local:daa_sim".to_string(),
            iam_endpoint_url: "http://0.0.0.0:8290".to_string(),
        })
        //.with_auth(AuthMethod::IbmCloudAppId {
        //    username: "demo".to_string(),
        //    password: "demopass".to_string(),
        //})
        .with_timeout(Duration::from_secs(60))
        .with_retry_policy(retry_policy)
        .build()
        .unwrap();

    let backends = client.list_backends::<Backends>().await?;
    println!("{}", serde_json::to_string_pretty(&backends).unwrap());

    for backend in backends.backends {
        println!("backend configuration for {}", backend.name);
        let config = client
            .get_backend_configuration::<serde_json::Value>(&backend.name)
            .await?;
        println!("{}", serde_json::to_string_pretty(&config).unwrap());

        println!("backend properties for {}", backend.name);
        let props = client
            .get_backend_properties::<serde_json::Value>(&backend.name)
            .await?;
        println!("{}", serde_json::to_string_pretty(&props).unwrap());

        println!("backend pulse defaults for {}", backend.name);
        let pulse_defaults = client
            .get_backend_pulse_defaults::<serde_json::Value>(&backend.name)
            .await?;
        println!("{}", serde_json::to_string_pretty(&pulse_defaults).unwrap());

        println!("backend details for {}", backend.name);
        let backend_details = client.get_backend::<Backend>(&backend.name).await?;
        println!(
            "{}",
            serde_json::to_string_pretty(&backend_details).unwrap()
        );
    }

    Ok(())
}
