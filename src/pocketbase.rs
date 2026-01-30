use std::sync::{Arc, Mutex};
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize};
use crate::auth::{AuthRequest, AuthResponse, AuthStore, DefaultAuthRecord, DefaultAuthResponseRecord};
use crate::error::ApiError;

#[derive(Clone)]
pub struct CollectionService {
    base_crud_path: &'static str,
    client: Arc<Client>,
    base_url: String,
}

pub struct Collection<T>
where T: DeserializeOwned + Clone {
    client: Arc<Client>,
    auth_store: Arc<Mutex<AuthStore<T>>>,
    base_url: String,
    collection_id_or_name: String,
}

pub struct PocketBase<T = DefaultAuthRecord>
where T: DeserializeOwned + Clone {
    auth_store: Arc<Mutex<AuthStore<T>>>,
    collections: CollectionService,
    client: Arc<Client>,
    base_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullListResponse<T> {
    pub items: Vec<T>,
    pub page: f64,
    pub per_page: f64,
    pub total_items: f64,
    pub total_pages: f64,
}

#[derive(Debug, Deserialize)]
pub struct ResponseError {
    pub message: String,
    pub status: u16,
}

impl<T> PocketBase<T>
where T: DeserializeOwned + Clone {
    pub fn base_url(&self) -> &String { &self.base_url }

    pub fn collections(&self) -> &CollectionService { &self.collections }

    pub fn auth_store(&self) -> &Mutex<AuthStore<T>> { &self.auth_store }

    pub fn new(base_url: impl Into<String>) -> Result<Self, ApiError> {
        let client = Arc::new(Client::new());
        let url = base_url.into().strip_suffix("/").unwrap().to_owned();
        Ok(Self {
            client: client.clone(),
            base_url: url.clone(),
            auth_store: Arc::new(Mutex::new(AuthStore::default())),
            collections: CollectionService {
                base_crud_path: "/api/collections",
                base_url: url.clone(),
                client: client.clone(),
            }
        })
    }

    pub fn collection(&self, name_or_id: impl Into<String>) -> Collection<T> {
        Collection {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            collection_id_or_name: name_or_id.into(),
            auth_store: self.auth_store.clone(),
        }
    }
}

impl PocketBase<DefaultAuthRecord> {
    pub fn default(base_url: impl Into<String>) -> Result<Self, ApiError> {
        PocketBase::new(base_url.into())
    }
}

impl<T> Collection<T>
where T: DeserializeOwned + Clone {
    async fn handle_response_body<E: DeserializeOwned>(&self, body: &String) -> Result<E, ApiError> {
        let response = serde_json::from_str::<E>(body);
        if response.is_ok() {
            Ok(response.unwrap())
        } else {
            let error = serde_json::from_str::<ResponseError>(body).unwrap();
            Err(ApiError::Http(StatusCode::from_u16(error.status).unwrap(), error.message))
        }
    }

    pub async fn get_full_list<E: DeserializeOwned>(&self) -> Result<FullListResponse<E>, ApiError> {
        let url = format!("{}/api/collections/{}/records", self.base_url, self.collection_id_or_name);
        let body = self.client.get(&url).send().await?.text().await?;
        self.handle_response_body(&body).await
    }

    pub async fn auth_with_password(&mut self, identity: impl Into<String>, password: impl Into<String>) -> Result<AuthResponse<T>, ApiError> {
        let url = format!("{}/api/collections/{}/auth-with-password", self.base_url, self.collection_id_or_name);
        let payload = AuthRequest {
            password: password.into(),
            identity: identity.into(),
        };
        let body = self.client.post(&url).header("Content-Type", "application/json").json(&payload).send().await?.text().await?;
        let tmp = self.handle_response_body::<AuthResponse<DefaultAuthResponseRecord>>(&body).await;
        let result = self.handle_response_body::<AuthResponse<T>>(&body).await;
        if let Ok(response) = &tmp {
            let token = response.token.clone();
            let mut lock = self.auth_store.lock().unwrap();
            lock.set_token(token);
            lock.set_collection(response.record.collection_id.clone(), response.record.collection_name.clone());
            if let Ok(actual_result) = &result {
            lock.set_record(actual_result.record.clone());
            }
        }
        result
    }
}

impl CollectionService {
    pub fn base_crud_path(&self) -> &'static str {
        self.base_crud_path
    }

    pub async fn get_full_list(&self) -> Result<String, ApiError> {
        // TODO: requires authentication header
        let url = format!("{}{}", self.base_url, self.base_crud_path);
        let body = self.client.get(url).send().await?.text().await?;
        Ok(body)
    }
}