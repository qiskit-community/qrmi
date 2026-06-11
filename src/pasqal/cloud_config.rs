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

use anyhow::{anyhow, Result};
use log::warn;
use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_PASQAL_CLOUD_AUTH_ENDPOINT: &str = "authenticate.pasqal.cloud/oauth/token";

#[derive(Debug, Clone, Default)]
pub(crate) struct PasqalConfig {
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
    pub(crate) token: Option<String>,
    pub(crate) project_id: Option<String>,
    pub(crate) auth_endpoint: Option<String>,
}

impl PasqalConfig {
    pub(crate) fn read(backend_name: &str) -> Result<Self> {
        read_pasqal_config(backend_name)
    }

    pub(crate) fn project_id(&self, backend_name: &str) -> Result<String> {
        let project_id_var = format!("{backend_name}_QRMI_PASQAL_CLOUD_PROJECT_ID");
        env::var(&project_id_var)
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or(self.project_id.clone().filter(|v| !v.trim().is_empty()))
            .or(read_qrmi_config_env_value(
                backend_name,
                "QRMI_PASQAL_CLOUD_PROJECT_ID",
            ))
            .ok_or_else(|| {
                anyhow!(
                    "{project_id_var} is not set and no project_id was found in ~/.pasqal/config or /etc/slurm/qrmi_config.json"
                )
            })
    }

    pub(crate) fn auth_token(&self, backend_name: &str) -> Option<String> {
        env_config_value(backend_name, "QRMI_PASQAL_CLOUD_AUTH_TOKEN")
            .or(self.token.clone().filter(|v| !v.trim().is_empty()))
            .or(read_qrmi_config_env_value(
                backend_name,
                "QRMI_PASQAL_CLOUD_AUTH_TOKEN",
            ))
    }

    pub(crate) fn auth_endpoint(&self, backend_name: &str) -> String {
        env_config_value(backend_name, "QRMI_PASQAL_CLOUD_AUTH_ENDPOINT")
            .or(self.auth_endpoint.clone().filter(|v| !v.trim().is_empty()))
            .or(read_qrmi_config_env_value(
                backend_name,
                "QRMI_PASQAL_CLOUD_AUTH_ENDPOINT",
            ))
            .unwrap_or_else(|| DEFAULT_PASQAL_CLOUD_AUTH_ENDPOINT.to_string())
    }

    pub(crate) fn base_url(&self, backend_name: &str) -> Option<String> {
        env_config_value(backend_name, "QRMI_PASQAL_CLOUD_BASE_URL").or(read_qrmi_config_env_value(
            backend_name,
            "QRMI_PASQAL_CLOUD_BASE_URL",
        ))
    }
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
                return Err(anyhow!("malformed environment variable in path: missing closing brace after ${{{key}}}"));
            }
            if key.is_empty()
                || !key
                    .chars()
                    .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
            {
                return Err(anyhow!(
                    "malformed environment variable in path: ${{{key}}}"
                ));
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

pub(crate) fn read_pasqal_config(backend_name: &str) -> Result<PasqalConfig> {
    let mut config_path_candidates: Vec<PathBuf> = Vec::new();
    let mut configured_config_root_paths: Vec<PathBuf> = Vec::new();

    if let Ok(config_root) = env::var("PASQAL_CONFIG_ROOT") {
        if let Some(path) = pasqal_config_path_from_root(&config_root)? {
            configured_config_root_paths.push(path.clone());
            config_path_candidates.push(path);
        }
    }

    for backend_config_root_var in [
        format!("{backend_name}_QRMI_PASQAL_CONFIG_ROOT"),
        format!("{backend_name}_PASQAL_CONFIG_ROOT"),
    ] {
        if let Ok(config_root) = env::var(&backend_config_root_var) {
            if let Some(path) = pasqal_config_path_from_root(&config_root)? {
                configured_config_root_paths.push(path.clone());
                config_path_candidates.push(path);
            }
        }
    }

    if let Some(config_root) = read_qrmi_config_env_value(backend_name, "QRMI_PASQAL_CONFIG_ROOT")
        .or_else(|| read_qrmi_config_env_value(backend_name, "PASQAL_CONFIG_ROOT"))
    {
        if let Some(path) = pasqal_config_path_from_root(&config_root)? {
            configured_config_root_paths.push(path.clone());
            config_path_candidates.push(path);
        }
    }

    if let Ok(home) = env::var("HOME") {
        if !home.trim().is_empty() {
            let mut path = PathBuf::from(home);
            path.push(".pasqal");
            path.push("config");
            config_path_candidates.push(path);
        }
    }

    let content = match config_path_candidates
        .iter()
        .find_map(|path| fs::read_to_string(path).ok())
    {
        Some(content) => content,
        None => {
            for path in configured_config_root_paths {
                warn!(
                    "Pasqal config root is set but config file was not found: {}",
                    path.display()
                );
            }
            return Ok(PasqalConfig::default());
        }
    };

    let mut config = PasqalConfig::default();

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

    Ok(config)
}

pub(crate) fn read_qrmi_config_env_value(backend_name: &str, key: &str) -> Option<String> {
    let content = fs::read_to_string("/etc/slurm/qrmi_config.json").ok()?;
    read_qrmi_config_env_value_from_content(&content, backend_name, key)
}

pub(crate) fn read_qrmi_config_env_value_from_content(
    content: &str,
    backend_name: &str,
    key: &str,
) -> Option<String> {
    let root: serde_json::Value = serde_json::from_str(content).ok()?;
    let resources = root.get("resources")?.as_array()?;

    for r in resources {
        let Some(name) = r.get("name").and_then(|name| name.as_str()) else {
            continue;
        };
        if name != backend_name {
            continue;
        }
        let env = r.get("environment")?.as_object()?;
        let v = env.get(key)?.as_str()?.trim();
        if v.is_empty() {
            return None;
        }
        return Some(v.to_string());
    }
    None
}

fn env_config_value(backend_name: &str, key: &str) -> Option<String> {
    env::var(format!("{backend_name}_{key}"))
        .ok()
        .filter(|v| !v.trim().is_empty())
}

pub(crate) fn resolve_pasqal_credentials(cfg: &PasqalConfig) -> (Option<String>, Option<String>) {
    let username = env::var("PASQAL_USERNAME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or(cfg.username.clone().filter(|v| !v.trim().is_empty()));
    let password = env::var("PASQAL_PASSWORD")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or(cfg.password.clone().filter(|v| !v.trim().is_empty()));
    (username, password)
}

pub(crate) fn resolve_pasqal_service_account_credentials(
    backend_name: &str,
    cfg: &PasqalConfig,
) -> (Option<String>, Option<String>) {
    let client_id = env_config_value(backend_name, "QRMI_PASQAL_CLOUD_CLIENT_ID")
        .or(cfg.client_id.clone().filter(|v| !v.trim().is_empty()))
        .or(read_qrmi_config_env_value(
            backend_name,
            "QRMI_PASQAL_CLOUD_CLIENT_ID",
        ));
    let client_secret = env_config_value(backend_name, "QRMI_PASQAL_CLOUD_CLIENT_SECRET")
        .or(cfg.client_secret.clone().filter(|v| !v.trim().is_empty()))
        .or(read_qrmi_config_env_value(
            backend_name,
            "QRMI_PASQAL_CLOUD_CLIENT_SECRET",
        ));
    (client_id, client_secret)
}
