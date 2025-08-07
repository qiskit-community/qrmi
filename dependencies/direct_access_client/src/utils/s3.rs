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

//! Helpers which provide minimum functionalities for operating S3 objects.

use anyhow::{bail, Result};
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::presigning::PresigningConfig;
use core::time::Duration;

/// A S3 client helper which provides minimum functionalities for operating S3 objects.
#[derive(Debug, Clone)]
pub struct S3Client {
    s3_client: aws_sdk_s3::Client,
}

impl S3Client {
    /// Construct a new [`S3Client`] with the specified S3 endpoint, AWS credentials
    /// and region.
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::utils::s3::S3Client;
    ///
    /// let _client = S3Client::new(
    ///     "http://localhost:9000",
    ///     "your_access_key",
    ///     "your_secret",
    ///     "your_region"
    /// );
    /// ```
    pub fn new(
        endpoint_url: impl Into<String>,
        aws_access_key_id: impl Into<String>,
        aws_secret_access_key: impl Into<String>,
        s3_region: impl Into<String>,
    ) -> Self {
        let cred = aws_credential_types::Credentials::new(
            aws_access_key_id.into(),
            aws_secret_access_key.into(),
            None,
            None,
            "direct_access_client",
        );

        let s3_config = aws_sdk_s3::config::Builder::new()
            .endpoint_url(endpoint_url.into())
            .credentials_provider(cred)
            .region(aws_sdk_s3::config::Region::new(s3_region.into()))
            .force_path_style(true)
            .build();

        Self {
            s3_client: aws_sdk_s3::Client::from_conf(s3_config),
        }
    }

    /// Returns the presigned URL for GET operation against the specified key in the S3 bucket
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::utils::s3::S3Client;
    ///
    /// let client = S3Client::new(
    ///     "http://localhost:9000",
    ///     "your_access_key",
    ///     "your_secret",
    ///     "your_region"
    /// );
    /// let _url = client.get_presigned_url_for_get("your_bucket", "obj_key", 3600);
    /// ```
    pub async fn get_presigned_url_for_get(
        &self,
        bucket_name: impl Into<String>,
        key_name: impl Into<String>,
        expires_in: u64,
    ) -> Result<String> {
        let presigned_config = PresigningConfig::expires_in(Duration::from_secs(expires_in))?;
        let presigned_url = match self
            .s3_client
            .get_object()
            .bucket(bucket_name)
            .key(key_name)
            .presigned(presigned_config)
            .await
        {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "An error occurred while generating the presigned URL: {}",
                    DisplayErrorContext(&err)
                ));
            }
        };
        Ok(presigned_url.uri().to_string())
    }

    /// Returns the presigned URL for PUT operation against the specified key in the S3 bucket
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::utils::s3::S3Client;
    ///
    /// let client = S3Client::new(
    ///     "http://localhost:9000",
    ///     "your_access_key",
    ///     "your_secret",
    ///     "your_region"
    /// );
    /// let _url = client.get_presigned_url_for_put("your_bucket", "obj_key", 3600);
    /// ```
    pub async fn get_presigned_url_for_put(
        &self,
        bucket_name: impl Into<String>,
        key_name: impl Into<String>,
        expires_in: u64,
    ) -> Result<String> {
        let presigned_config = PresigningConfig::expires_in(Duration::from_secs(expires_in))?;
        let presigned_url = match self
            .s3_client
            .put_object()
            .bucket(bucket_name)
            .key(key_name)
            .presigned(presigned_config)
            .await
        {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "An error occurred while generating the presigned URL: {}",
                    DisplayErrorContext(&err)
                ));
            }
        };
        Ok(presigned_url.uri().to_string())
    }

    /// Adds an object to a bucket.
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::utils::s3::S3Client;
    ///
    /// let client = S3Client::new(
    ///     "http://localhost:9000",
    ///     "your_access_key",
    ///     "your_secret",
    ///     "your_region"
    /// );
    ///
    /// let content = String::from("Hello, World.");
    /// client.put_object("your_bucket", "obj_key", content.as_bytes());
    /// ```
    pub async fn put_object(
        &self,
        bucket_name: impl Into<String>,
        key_name: impl Into<String>,
        content: &[u8],
    ) -> Result<()> {
        let _ = match self
            .s3_client
            .put_object()
            .bucket(bucket_name)
            .key(key_name)
            .body(content.to_vec().into())
            .send()
            .await
        {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "An error occurred while adding an object to S3 bucket: {}",
                    DisplayErrorContext(&err)
                ));
            }
        };
        Ok(())
    }

    /// Retrieves an object from a bucket.
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::utils::s3::S3Client;
    ///
    /// let client = S3Client::new(
    ///     "http://localhost:9000",
    ///     "your_access_key",
    ///     "your_secret",
    ///     "your_region"
    /// );
    ///
    /// let content = client.get_object("your_bucket", "obj_key");
    /// ```
    pub async fn get_object(
        &self,
        bucket_name: impl Into<String>,
        key_name: impl Into<String>,
    ) -> Result<Vec<u8>> {
        let mut object = match self
            .s3_client
            .get_object()
            .bucket(bucket_name)
            .key(key_name)
            .send()
            .await
        {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "An error occurred while retrieving an object from S3 bucket: {}",
                    DisplayErrorContext(&err)
                ));
            }
        };

        let mut data = Vec::<u8>::new();
        while let Some(bytes) = object.body.try_next().await? {
            data.append(&mut bytes.to_vec());
        }
        Ok(data)
    }

    /// Deletes an object from a bucket.
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::utils::s3::S3Client;
    ///
    /// let client = S3Client::new(
    ///     "http://localhost:9000",
    ///     "your_access_key",
    ///     "your_secret",
    ///     "your_region"
    /// );
    ///
    /// client.delete_object("your_bucket", "obj_key");
    /// ```
    pub async fn delete_object(
        &self,
        bucket_name: impl Into<String>,
        key_name: impl Into<String>,
    ) -> Result<()> {
        let _ = match self
            .s3_client
            .delete_object()
            .bucket(bucket_name)
            .key(key_name)
            .send()
            .await
        {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "An error occurred while deleting an object from S3 bucket: {}",
                    DisplayErrorContext(&err)
                ));
            }
        };
        Ok(())
    }

    /// Lists object names available in a bucket.
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::utils::s3::S3Client;
    ///
    /// let client = S3Client::new(
    ///     "http://localhost:9000",
    ///     "your_access_key",
    ///     "your_secret",
    ///     "your_region"
    /// );
    ///
    /// let objects = client.list_objects("your_bucket");
    /// ```
    pub async fn list_objects(&self, bucket_name: impl Into<String>) -> Result<Vec<String>> {
        let mut key_names = Vec::<String>::new();
        let mut cont_token = None;

        let bucket: String = bucket_name.into();

        loop {
            match self
                .s3_client
                .list_objects_v2()
                .bucket(bucket.clone())
                .set_continuation_token(cont_token.to_owned())
                .send()
                .await
            {
                Ok(resp) => {
                    for object in resp.contents() {
                        key_names.push(object.key().unwrap_or_default().to_string());
                    }
                    if let Some(is_truncated) = resp.is_truncated {
                        if !is_truncated {
                            break;
                        }
                        cont_token = resp.next_continuation_token().map(|s| s.to_string());
                    } else {
                        break;
                    }
                }
                Err(err) => {
                    bail!(format!(
                        "An error occurred while listing objects in S3 bucket: {}",
                        DisplayErrorContext(&err)
                    ));
                }
            }
        }
        Ok(key_names)
    }
}
