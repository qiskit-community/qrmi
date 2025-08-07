//
// (C) Copyright IBM 2024
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use direct_access_api::ClientBuilder;

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = ClientBuilder::new("http://0.0.0.0:8290").build().unwrap();

    let version = client.get_service_version().await?;
    println!("{}", version);

    Ok(())
}
