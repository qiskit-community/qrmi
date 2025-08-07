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

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct GetAccessTokenResponse {
    // The IAM access token that can be used to invoke Direct Access API. Use this token with the prefix Bearer in the HTTP header Authorization for invocations of Direct Access API.
    pub access_token: String,
    // Number of seconds until the IAM access token will expire.
    pub expires_in: u64,
    // Number of seconds counted since January 1st, 1970, until the IAM access token will expire. Only available in the responses from IAM.
    pub expiration: Option<u64>,
    // The type of the token. Currently, only Bearer is returned.
    pub token_type: String,
    // (optional) A refresh token that can be used to get a new IAM access token if that token is expired. Only available in the response from IAM
    pub refresh_token: Option<String>,
    // (optional) Only available in the response from IAM
    pub ims_user_id: Option<u64>,
    // (optional) Only available in the response from IAM
    pub scope: Option<String>,
}
