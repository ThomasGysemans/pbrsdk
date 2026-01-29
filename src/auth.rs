use serde::de::DeserializeOwned;
use pbrsdk_macros::base_system_fields;

#[base_system_fields]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultAuthRecord {
    email: String,
    verified: bool,
    email_visibility: bool,
    created: String,
    updated: String,
    name: String,
}

#[derive(Debug)]
pub struct AuthStore<T = DefaultAuthRecord>
where T: DeserializeOwned {
    token: Option<String>,
    record: Option<T>,
    collection_name: Option<String>,
    collection_id: Option<String>,
}

impl<T> Default for AuthStore<T>
where T: DeserializeOwned {
    fn default() -> Self {
        AuthStore {
            token: None,
            record: None,
            collection_id: None,
            collection_name: None,
        }
    }
}

impl<T> AuthStore<T>
where T: DeserializeOwned {
    pub fn token(&self) -> Option<String> {
        self.token.clone()
    }

    pub fn record(&self) -> Option<&T> {
        self.record.as_ref()
    }

    pub fn is_valid(&self) -> bool {
        self.token.is_some() && self.record.is_some() && self.collection_id.is_some() && self.collection_name.is_some()
    }

    pub fn is_superuser(&self) -> bool {
        self.is_valid() && self.collection_name.as_ref().unwrap() == "_superusers"
    }
}