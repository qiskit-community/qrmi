// This code is part of Qiskit.
//
// (C) Copyright Pasqal 2025, 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::common::resolve_opt;
use crate::{QrmiError, Result};
use log::{debug, warn};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_PASQAL_CLOUD_AUTH_ENDPOINT: &str = "authenticate.pasqal.cloud/oauth/token";

#[derive(Debug, Clone, Default)]
pub(crate) struct PasqalCloudConfig {
    // Config stores the eventual hashmap from the [`crate::PasqalCloud::from_config`]
    pub config: Option<HashMap<String, String>>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
    pub(crate) token: Option<String>,
    pub(crate) project_id: Option<String>,
    pub(crate) auth_endpoint: Option<String>,
}

impl PasqalCloudConfig {
    pub(crate) fn from_opt(
        backend_name: &str,
        config: Option<&HashMap<String, String>>,
    ) -> Result<Self> {
        read_pasqal_config(backend_name, config)
    }

    pub(crate) fn project_id(&self, backend_name: &str) -> Option<String> {
        resolve_backend_prefixed_param(
            backend_name,
            "QRMI_PASQAL_CLOUD_PROJECT_ID",
            self.config.as_ref(),
        )
        .or(self.project_id.clone().filter(|v| !v.trim().is_empty()))
    }

    pub(crate) fn auth_token(&self, backend_name: &str) -> Option<String> {
        resolve_backend_prefixed_param(
            backend_name,
            "QRMI_PASQAL_CLOUD_AUTH_TOKEN",
            self.config.as_ref(),
        )
        .or(self.token.clone().filter(|v| !v.trim().is_empty()))
    }

    pub(crate) fn auth_endpoint(&self, backend_name: &str) -> String {
        resolve_backend_prefixed_param(
            backend_name,
            "QRMI_PASQAL_CLOUD_AUTH_ENDPOINT",
            self.config.as_ref(),
        )
        .or(self.auth_endpoint.clone().filter(|v| !v.trim().is_empty()))
        .unwrap_or_else(|| DEFAULT_PASQAL_CLOUD_AUTH_ENDPOINT.to_string())
    }

    pub(crate) fn base_url(&self, backend_name: &str) -> Option<String> {
        resolve_backend_prefixed_param(
            backend_name,
            "QRMI_PASQAL_CLOUD_BASE_URL",
            self.config.as_ref(),
        )
    }

    pub(crate) fn credentials(&self) -> (Option<String>, Option<String>) {
        let username = resolve_opt("PASQAL_USERNAME", self.config.as_ref())
            .filter(|v| !v.trim().is_empty())
            .or(self.username.clone().filter(|v| !v.trim().is_empty()));
        let password = resolve_opt("PASQAL_PASSWORD", self.config.as_ref())
            .filter(|v| !v.trim().is_empty())
            .or(self.password.clone().filter(|v| !v.trim().is_empty()));
        (username, password)
    }

    pub(crate) fn service_account_credentials(
        &self,
        backend_name: &str,
    ) -> (Option<String>, Option<String>) {
        let client_id = resolve_backend_prefixed_param(
            backend_name,
            "QRMI_PASQAL_CLOUD_CLIENT_ID",
            self.config.as_ref(),
        )
        .or(self.client_id.clone().filter(|v| !v.trim().is_empty()));
        let client_secret = resolve_backend_prefixed_param(
            backend_name,
            "QRMI_PASQAL_CLOUD_CLIENT_SECRET",
            self.config.as_ref(),
        )
        .or(self.client_secret.clone().filter(|v| !v.trim().is_empty()));
        (client_id, client_secret)
    }
}

// Env vars are per-instance (`<backend_name>_...`); config map keys use the
// same name minus that prefix, since a config map is already scoped to one
// backend.
fn resolve_backend_prefixed_param(
    backend_name: &str,
    value: &str,
    config: Option<&HashMap<String, String>>,
) -> Option<String> {
    let prefix = if config.is_some() {
        String::new()
    } else {
        format!("{backend_name}_")
    };
    resolve_opt(&format!("{prefix}{value}"), config).filter(|v| !v.trim().is_empty())
}

fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

pub(crate) fn pasqal_config_path_from_root(config_root: &str) -> Result<Option<PathBuf>> {
    let config_root = config_root.trim();
    if config_root.is_empty() {
        return Ok(None);
    }
    let expanded_config_root = expand_config_root(config_root)?;
    let mut path = PathBuf::from(expanded_config_root);
    path.push(".pasqal");
    path.push("config");
    Ok(Some(path))
}

// Expands ~ to the user's home directory and also expands environment variables in the config root path.
fn expand_config_root(config_root: &str) -> Result<String> {
    let home_expanded = match config_root {
        "~" => env::var("HOME").unwrap_or_else(|_| config_root.to_string()),
        root if root.starts_with("~/") => env::var("HOME")
            .map(|home| format!("{home}{}", &root[1..]))
            .unwrap_or_else(|_| config_root.to_string()),
        _ => config_root.to_string(),
    };

    expand_env_vars(&home_expanded)
}

// Expand environment variables in path strings. Supports $VAR, ${VAR}, and $$.
pub(crate) fn expand_env_vars(value: &str) -> Result<String> {
    let mut expanded = String::new();
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        // If it's not a $, just add it to the result.
        if ch != '$' {
            expanded.push(ch);
            continue;
        }

        // Handle $$ -> literal $
        if chars.peek() == Some(&'$') {
            chars.next();
            expanded.push('$');
            continue;
        }

        // Support {VAR} syntax. Verify that closing brace exists.
        if chars.peek() == Some(&'{') {
            chars.next();
            let mut key = String::new();
            let mut closed = false;
            for var_ch in chars.by_ref() {
                if var_ch == '}' {
                    closed = true;
                    break;
                }
                key.push(var_ch);
            }

            if !closed {
                return Err(QrmiError::InvalidInput(format!(
                    "malformed environment variable in path: missing closing brace after ${{{key}}}"
                )));
            }
            if key.is_empty()
                || !key
                    .chars()
                    .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
            {
                return Err(QrmiError::InvalidInput(format!(
                    "malformed environment variable in path: ${{{key}}}"
                )));
            }

            if let Ok(var_value) = env::var(&key) {
                expanded.push_str(&var_value);
            }
            continue;
        } // end support for ${VAR} syntax

        // Support $VAR syntax. Read until non-alphanumeric and non-underscore character.
        let mut key = String::new();
        while let Some(var_ch) = chars.peek().copied() {
            if var_ch == '_' || var_ch.is_ascii_alphanumeric() {
                key.push(var_ch);
                chars.next();
            } else {
                break;
            }
        }

        if key.is_empty() {
            expanded.push('$');
        } else if let Ok(var_value) = env::var(&key) {
            expanded.push_str(&var_value);
        }
    }

    Ok(expanded)
}

pub(crate) fn read_pasqal_config(
    backend_name: &str,
    hashmap_config: Option<&HashMap<String, String>>,
) -> Result<PasqalCloudConfig> {
    // A config map is already scoped to one backend, so it only has the
    // single, unprefixed `PASQAL_CONFIG_ROOT` key. Env vars additionally
    // fall back to the backend-prefixed override if the global one isn't set.
    let mut config_root_path = match resolve_opt("PASQAL_CONFIG_ROOT", hashmap_config) {
        Some(config_root) => pasqal_config_path_from_root(&config_root)?,
        None => None,
    };
    if config_root_path.is_none() && hashmap_config.is_none() {
        config_root_path = match resolve_opt(&format!("{backend_name}_PASQAL_CONFIG_ROOT"), None) {
            Some(config_root) => pasqal_config_path_from_root(&config_root)?,
            None => None,
        };
    }
    let home_config_path = match resolve_opt("HOME", hashmap_config) {
        Some(home) => pasqal_config_path_from_root(&home)?,
        None => None,
    };

    // Explicit "PASQAL_CONFIG_ROOT" will be tried before the "HOME" fallback
    let mut config_path_candidates = Vec::new();
    if let Some(path) = config_root_path.clone() {
        config_path_candidates.push(path);
    }
    if let Some(path) = home_config_path {
        config_path_candidates.push(path);
    }

    let mut config = resolve_pasqal_config(&config_path_candidates, config_root_path.as_ref());
    // Store the eventual hashmap config here
    config.config = hashmap_config.cloned();

    Ok(config)
}

// Loads the config from the first readable path in `candidates`. If none is readable and
// `explicit_root` was set, warns that the explicitly configured root had no config file.
fn resolve_pasqal_config(
    candidates: &[PathBuf],
    explicit_root: Option<&PathBuf>,
) -> PasqalCloudConfig {
    match candidates.iter().find_map(load_pasqal_config_file) {
        Some(config) => config,
        None => {
            if let Some(path) = explicit_root {
                warn!(
                    "Pasqal config root is set but no config file was found. Checked: {}",
                    path.display()
                );
            }
            PasqalCloudConfig::default()
        }
    }
}

// Reads and parses the Pasqal config file at `path`, or returns `None` if it can't be read.
fn load_pasqal_config_file(path: &PathBuf) -> Option<PasqalCloudConfig> {
    let content = fs::read_to_string(path).ok()?;
    debug!("Reading Pasqal config file: {}", path.display());
    Some(parse_pasqal_config_content(&content))
}

fn parse_pasqal_config_content(content: &str) -> PasqalCloudConfig {
    let mut config = PasqalCloudConfig::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let (k, v) = match line.split_once('=') {
            Some((k, v)) => (k.trim(), strip_quotes(v).trim()),
            None => continue,
        };
        if k.is_empty() {
            continue;
        }

        match k.to_ascii_lowercase().as_str() {
            "username" => config.username = Some(v.to_string()),
            "password" => config.password = Some(v.to_string()),
            "client_id" => config.client_id = Some(v.to_string()),
            "client_secret" => config.client_secret = Some(v.to_string()),
            "token" => config.token = Some(v.to_string()),
            "project_id" => config.project_id = Some(v.to_string()),
            "auth_endpoint" => config.auth_endpoint = Some(v.to_string()),
            _ => {}
        }
    }

    config
}
