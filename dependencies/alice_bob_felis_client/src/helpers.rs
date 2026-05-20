
use base64::{engine::general_purpose, Engine};

// Felis API keys are basic auth credentials in disguise.
// We need to decode them to pass them to our client in the format it expects.
// The client will then reencode.
#[allow(clippy::type_complexity)]
pub fn decode_api_key(encoded: &str) -> Result<Option<(String, Option<String>)>, Box<dyn std::error::Error>> {
    // Decode from Base64
    let decoded_bytes = general_purpose::STANDARD.decode(encoded)?;
    let decoded_str = String::from_utf8(decoded_bytes)?;

    // Split at the first ':'
    let mut parts = decoded_str.splitn(2, ':');

    let username = parts
        .next()
        .ok_or("missing username part")?
        .to_string();

    let password = parts
        .next()
        .ok_or("missing password part")?
        .to_string();

    Ok(Some((username, Some(password))))
}
