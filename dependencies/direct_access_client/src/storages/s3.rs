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

use anyhow::{bail, Result};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use core::time::Duration;

use aws_sdk_s3::error::ProvideErrorMetadata;

/// Returns S3 presigned URL for GET operation
///
/// # Arguments
/// * `client` - S3 Client.
/// * `bucket_name` - S3 bucket name.
/// * `key` - S3 object key.
///     
pub async fn get_presigned_url(
    client: &Client,
    bucket_name: &str,
    key: &str,
    expiration: u64,
) -> Result<String> {
    let expires_in = Duration::from_secs(expiration);
    let presigned_config = PresigningConfig::expires_in(expires_in)?;
    let presigned_url = match client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .presigned(presigned_config)
        .await
    {
        Ok(val) => val,
        Err(err) => {
            let err = err.into_service_error();
            bail!(format!("{}", err.message().unwrap()));
        }
    };
    Ok(presigned_url.uri().to_string())
}

/// Returns S3 presigned URL for PUT operation
///
/// # Arguments
/// * `s3params` - S3ConnectionParams.
/// * `bucket_name` - S3 bucket name.
/// * `obj_key` - S3 object key.
///
pub async fn get_presigned_url_for_put(
    client: &Client,
    bucket_name: &str,
    obj_key: &str,
    expiration: u64,
) -> Result<String> {
    let expires_in = Duration::from_secs(expiration);
    let presigned_config = PresigningConfig::expires_in(expires_in)?;
    let presigned_url = match client
        .put_object()
        .bucket(bucket_name)
        .key(obj_key)
        .presigned(presigned_config)
        .await
    {
        Ok(val) => val,
        Err(err) => {
            let err = err.into_service_error();
            bail!(format!("{}", err.message().unwrap()));
        }
    };
    Ok(presigned_url.uri().to_string())
}
