use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use pbrsdk_macros::base_system_fields;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::ApiError;

#[base_system_fields]
#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DefaultAuthRecord {
    pub email: String,
    pub verified: bool,
    pub email_visibility: bool,
    pub created: String,
    pub updated: String,
    pub name: Option<String>, // it's optional because such column doesn't exist in the default _superusers collection
}

#[derive(Debug)]
pub struct AuthStore<T>
where T: DeserializeOwned + Clone {
    token: Option<String>,
    record: Option<T>,
    collection_name: Option<String>,
    collection_id: Option<String>,
}

impl<T> Default for AuthStore<T>
where T: DeserializeOwned + Clone {
    fn default() -> Self {
        AuthStore {
            token: None,
            record: None,
            collection_id: None,
            collection_name: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse<T> {
    pub record: T,
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DefaultAuthResponseRecord {
    pub collection_id: String,
    pub collection_name: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AuthRequest {
    pub identity: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JwtPayload {
    #[serde(rename = "type")]
    token_type: String,
    #[serde(rename = "collectionId")]
    collection_id: String,
    refreshable: bool,
    id: String,
    exp: u64, // expiration in seconds
}

impl<T> AuthStore<T>
where T: DeserializeOwned + Clone {
    pub fn token(&self) -> Option<String> {
        self.token.clone()
    }

    pub fn record(&self) -> Option<&T> {
        self.record.as_ref()
    }

    pub(crate) fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub(crate) fn set_record(&mut self, record: T) {
        self.record = Some(record);
    }

    pub(crate) fn set_collection(&mut self, collection_name: String, collection_id: String) {
        self.collection_name = Some(collection_name);
        self.collection_id = Some(collection_id);
    }

    pub(crate) fn is_some(&self) -> bool {
        self.token.is_some() && self.record.is_some() && self.collection_id.is_some() && self.collection_name.is_some()
    }

    pub fn is_valid(&self) -> bool {
        self.is_some() && !is_token_expired(self.token.as_ref().unwrap())
    }

    pub fn is_superuser(&self) -> bool {
        if !self.is_some() { return false; }
        let payload = get_token_payload(self.token.as_ref().unwrap());
        if let Ok(payload) = payload {
            return payload.token_type == "auth" && (self.collection_name.as_ref().unwrap() == "_superusers" || payload.collection_id == "pbc_3142635823");
        }
        false
    }
}

pub(crate) fn get_token_payload(token: &String) -> Result<JwtPayload, ApiError> {
    let payload = token.split('.').nth(1).ok_or("Invalid token");
    if let Ok(payload) = payload {
        let decoded = URL_SAFE_NO_PAD.decode(payload);
        if let Ok(decoded) = decoded {
            let decoded_str = String::from_utf8(decoded);
            if let Ok(decoded_str) = decoded_str {
                let json = serde_json::from_str::<JwtPayload>(&decoded_str);
                if let Ok(json) = json {
                    return Ok(json);
                }
            }
        }
    }
    Err(ApiError::Jwt())
}

pub(crate) fn is_token_expired(token: &String) -> bool {
    let payload = get_token_payload(token);
    if let Ok(payload) = payload {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        if payload.exp > timestamp {
            return false;
        }
    }
    true
}