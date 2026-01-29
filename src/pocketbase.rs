use std::sync::Arc;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use crate::error::ApiError;

pub struct CollectionService {
    pub base_crud_path: &'static str,
    pub client: Arc<Client>,
    pub base_url: String,
}

pub struct PocketBase {
    pub collections: CollectionService,
    pub client: Arc<Client>,
    pub base_url: String,
}

pub struct Collection {
    pb: PocketBase,
    pub collection_id_or_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response<T> {
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

impl PocketBase {
    pub fn new(base_url: impl Into<String>) -> Result<Self, ApiError> {
        let client = Arc::new(Client::new());
        let url = base_url.into();
        Ok(Self {
            client: client.clone(),
            base_url: url.clone(),
            collections: CollectionService {
                base_crud_path: "/api/collections",
                base_url: url.clone(),
                client: client.clone(),
            }
        })
    }

    pub fn collection(self, name_or_id: impl Into<String>) -> Collection {
        Collection {
            pb: self,
            collection_id_or_name: name_or_id.into(),
        }
    }
}

impl Collection {
    pub async fn get_full_list<T: DeserializeOwned>(self) -> Result<Response<T>, ApiError> {
        let url = format!("{}/api/collections/{}/records", self.pb.base_url, self.collection_id_or_name);
        let body = self.pb.client.get(url).send().await?.text().await?;
        let response = serde_json::from_str::<Response<T>>(&body);
        if response.is_ok() {
            Ok(response.unwrap())
        } else {
            let error = serde_json::from_str::<ResponseError>(&body).unwrap();
            Err(ApiError::Http(StatusCode::from_u16(error.status).unwrap(), error.message))
        }
    }
}

impl CollectionService {
    pub async fn get_full_list(self) -> Result<String, ApiError> {
        // TODO: requires authentication header
        let url = format!("{}{}", self.base_url, self.base_crud_path);
        let body = self.client.get(url).send().await?.text().await?;
        Ok(body)
    }
}