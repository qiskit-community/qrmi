use serde::{de::Error as _};
use generated::{apis::ResponseContent};
use generated::apis::{Error, configuration};
use generated::apis::jobs_service::UploadInputError;
use reqwest::multipart::{Part};

/// Internal use only
/// A content type supported by this client.
#[allow(dead_code)]
enum ContentType {
    Json,
    Text,
    Unsupported(String)
}

impl From<&str> for ContentType {
    fn from(content_type: &str) -> Self {
        if content_type.starts_with("application") && content_type.contains("json") {
            Self::Json
        } else if content_type.starts_with("text/plain") {
            Self::Text
        } else {
            Self::Unsupported(content_type.to_string())
        }
    }
}

/// Upload the input code for a job in QIR format to trigger its execution (provided that the target is available).
pub async fn upload_input(configuration: &configuration::Configuration, job_id: &str, input: String, authorization: Option<&str>) -> Result<serde_json::Value, Error<UploadInputError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_job_id = job_id;
    let p_header_authorization = authorization;

    let uri_str = format!("{}/v1/jobs/{job_id}/input", configuration.base_path, job_id=crate::apis::urlencode(p_path_job_id));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(param_value) = p_header_authorization {
        req_builder = req_builder.header("authorization", param_value.to_string());
    }
    if let Some(ref auth_conf) = configuration.basic_auth {
        req_builder = req_builder.basic_auth(auth_conf.0.to_owned(), auth_conf.1.to_owned());
    };

    let part = Part::bytes(input.into_bytes())
        .file_name("input")                 // REQUIRED to mimic Python
        .mime_str("application/octet-stream")?; // matches Python behavior
    let multipart_form = reqwest::multipart::Form::new().part("input", part);
    // TODO: support file upload for 'input' parameter
    req_builder = req_builder.multipart(multipart_form);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `serde_json::Value`"))),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `serde_json::Value`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<UploadInputError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}