use std::borrow::Cow;
use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use urlencoding::decode;

#[derive(Debug, Deserialize)]
pub struct Cookie {
    pub name: Option<String>,
    pub value: Option<String>,
    #[serde(rename = "Path")]
    pub path: Option<String>,
    #[serde(rename = "Expires")]
    pub expires: Option<String>,
    #[serde(rename = "SameSite")]
    pub same_site: Option<String>,
    #[serde(rename = "HttpOnly")]
    #[serde(deserialize_with = "string_to_bool")]
    pub http_only: bool,
    #[serde(rename = "Secure")]
    #[serde(deserialize_with = "string_to_bool")]
    pub secure: bool,
}

impl Default for Cookie {
    fn default() -> Self {
        Self {
            name: None,
            value: None,
            expires: None,
            path: None,
            same_site: None,
            http_only: false,
            secure: false,
        }
    }
}

/// This is used in macros only, in the [Cookie] struct.
#[allow(unused)]
fn string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::custom("expected \"true\" or \"false\"")),
    }
}

pub(crate) fn cookie_parse(str: &String) -> Result<Cookie, ()> {
    let mut map = HashMap::<String, String>::from_iter(str.split(';').map(|x| {
        let has_equal = x.contains("=");
        return if has_equal {
            let eq_idx = x.find("=").unwrap();
            let sliced_key = (&x[0..eq_idx]).trim().to_string();
            let key = if sliced_key == "pb_auth" { "value".to_string() } else { sliced_key };
            let value = (&x[eq_idx + 1..]).trim().to_string();
            if let Some(first_char) = x.chars().next() {
                if let Some(last_char) = x.chars().last() {
                    if (first_char == last_char) && first_char == '"' {
                        let unquoted_value = (&x[1..value.chars().count() - 1]).to_string();
                        let decoded_value = decode(&unquoted_value).unwrap_or(Cow::Borrowed(&unquoted_value));
                        return (key, decoded_value.to_string());
                    }
                }
            }
            (key, decode(&value).unwrap_or(Cow::Borrowed(&value)).to_string())
        } else {
            (x.trim().to_string(), String::new())
        }
    }));
    if map.contains_key("value") && !map["value"].is_empty() { map.insert("name".into(), "pb_auth".into()); }
    if map.contains_key("HttpOnly") && map["HttpOnly"].is_empty() { map.insert("HttpOnly".into(), "false".into()); }
    if map.contains_key("Secure") && map["Secure"].is_empty() { map.insert("Secure".into(), "false".into()); }
    Ok(serde_json::from_value(serde_json::to_value(map).unwrap()).unwrap())
}