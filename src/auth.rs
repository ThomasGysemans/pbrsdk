use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use pbrsdk_macros::base_system_fields;

#[base_system_fields]
#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DefaultAuthRecord {
    pub email: String,
    pub verified: bool,
    pub email_visibility: bool,
    pub created: String,
    pub updated: String,
    pub name: String,
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
pub struct DefaultAuthResponseRecord {
    pub collection_id: String,
    pub collection_name: String,
}

#[derive(Debug, Serialize)]
pub struct AuthRequest {
    pub identity: String,
    pub password: String,
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

    pub fn is_valid(&self) -> bool {
        self.token.is_some() && self.record.is_some() && self.collection_id.is_some() && self.collection_name.is_some()
    }

    pub fn is_superuser(&self) -> bool {
        self.is_valid() && self.collection_name.as_ref().unwrap() == "_superusers"
    }
}