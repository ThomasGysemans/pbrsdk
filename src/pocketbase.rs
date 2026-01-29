use std::sync::Arc;
use reqwest::Client;
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
    pub name: String,
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

    pub fn collection(self, name: impl Into<String>) -> Result<Collection, ApiError> {
        Ok(Collection {
            pb: self,
            name: name.into(),
        })
    }
}

impl Collection {
    pub async fn get_full_list<T: DeserializeOwned>(self) -> Result<Response<T>, ApiError> {
        let url = format!("{}/api/collections/{}/records", self.pb.base_url, self.name);
        let body = self.pb.client.get(url).send().await?.text().await?;
        let response: Response<T> = serde_json::from_str(&body).unwrap();
        Ok(response)
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