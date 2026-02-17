use std::sync::{Arc};
use reqwest::{StatusCode};
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use urlencoding::encode;
use crate::error::{ApiError};
use crate::auth::{AuthRequestPayload, AuthResponse, DefaultAuthResponseRecord};
use crate::common::{ResponseError, ViewOptions, ListOptions};
use crate::pocketbase::PocketBaseRef;

/// The server's response when requesting a list of records.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T> {
    /// The records themselves.
    pub items: Vec<T>,
    /// The current page.
    /// The server always respond with batches of records,
    /// the size of a single batch is defined by the `per_page` property.
    pub page: u64,
    /// The number of items in a page.
    /// If the number of items is equal to this number,
    /// then it either means there is no more records to fetch,
    /// or there's still at least one page left to fetch.
    pub per_page: u64,
    /// The total number of items.
    /// This number isn't necessarily identical to the items' length,
    /// since the latter is limited by the "per_page" property.
    pub total_items: i64,
    /// The total number of pages.
    pub total_pages: i64,
}

/// The service responsible for fetching records.
pub struct RecordService<T>
where T: DeserializeOwned + Clone {
    pub(crate) collection_id_or_name: String,
    pub(crate) pb: Arc<PocketBaseRef<T>>,
}

#[derive(Deserialize, Debug)]
struct RecordIdOnly {
    id: String,
}

impl<T> RecordService<T>
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

    fn get_auth_headers(&self) -> HeaderMap {
        let store = self.pb.auth_store.lock();
        let token = store.as_ref().unwrap().token.clone();
        let mut headers: HeaderMap = HeaderMap::new();
        if let Some(token) = token {
            headers.insert("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        headers
    }

    /// Fetches pages of records.
    pub async fn get_list<E: DeserializeOwned>(&self, options: ListOptions) -> Result<ListResponse<E>, ApiError> {
        let url = format!("{}/api/collections/{}/records{}", self.pb.base_url, self.collection_id_or_name, options.to_url_query());
        let headers = self.get_auth_headers();
        let body = self.pb.client
            .get(&url)
            .headers(headers)
            .send().await?
            .text().await?;
        self.handle_response_body(&body).await
    }

    /// Fetches one record based on its ID, which must exist.
    /// If the ID isn't found, the server will return a 404 error.
    pub async fn get_one<E: DeserializeOwned>(&self, id: impl Into<String>, options: Option<ViewOptions>) -> Result<E, ApiError> {
        let url = format!("{}/api/collections/{}/records/{}{}", self.pb.base_url, self.collection_id_or_name, encode(&id.into()), options.unwrap_or_default().to_url_query());
        let headers = self.get_auth_headers();
        let body = self.pb.client
            .get(&url)
            .headers(headers)
            .send().await?
            .text().await?;
        self.handle_response_body(&body).await
    }

    /// Gets the full list of records from the collection.
    pub async fn get_full_list<E: DeserializeOwned>(&self) -> Result<Vec<E>, ApiError> {
        let mut page_index = 1u64;
        let mut items: Vec<E> = Vec::new();
        loop {
            let pages = self.get_list::<E>(ListOptions::paginated_and_skip(page_index, 1000)).await;
            if let Err(err) = pages {
                return Err(err);
            }
            if let Ok(mut page) = pages {
                let number_of_fetched_items = page.items.len();
                items.append(&mut page.items);
                if number_of_fetched_items == page.per_page as usize {
                    page_index += 1;
                } else {
                    break;
                }
            }
        }
        Ok(items)
    }

    /// Returns the first found item by the specified filter.
    /// This is equivalent to calling `get_list()` with options "page" and "per_page" set to 1,
    /// then "skip_total" set to "false" and passing along the filter.
    ///
    /// For consistency with `get_one()`, this method will throw a 404 if the item isn't found.
    pub async fn get_first_list_item<E: DeserializeOwned>(&self, filter: impl Into<String>, options: Option<ViewOptions>) -> Result<E, ApiError> {
        let list_options = ListOptions::from_view(Some(1), Some(1), Some(filter.into()), options);
        let page = self.get_list::<E>(list_options).await;
        if let Err(err) = page {
            return Err(err);
        }
        if let Ok(mut page) = page {
            return Ok(page.items.pop().unwrap());
        }
        Err(ApiError::Http(StatusCode::NOT_FOUND, "There is no record matching the filter.".to_string()))
    }

    /// Creates a new item and returns the new record.
    pub async fn create<E: DeserializeOwned, S: Serialize>(&self, body: S, options: Option<ViewOptions>) -> Result<E, ApiError> {
        let url = format!("{}/api/collections/{}/records{}", self.pb.base_url, self.collection_id_or_name, options.unwrap_or_default().to_url_query());
        let headers = self.get_auth_headers();
        let body = self.pb.client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send().await?
            .text().await?;
        self.handle_response_body(&body).await
    }

    /// Deletes an existing item by its id.
    /// Returns nothing if the operation succeeds.
    pub async fn delete(&self, id: impl Into<String>) -> Result<(), ApiError> {
        let url = format!("{}/api/collections/{}/records/{}", self.pb.base_url, self.collection_id_or_name, encode(&id.into()));
        let headers = self.get_auth_headers();
        let body = self.pb.client
            .delete(&url)
            .headers(headers)
            .send().await?
            .text().await?;
        if body.is_empty() {
            Ok(())
        } else {
            let error = serde_json::from_str::<ResponseError>(&body);
            if let Ok(error) = error {
                Err(ApiError::Http(StatusCode::from_u16(error.status).unwrap(), error.message))
            } else {
                Ok(())
            }
        }
    }

    /// Updates an existing item by its ID.
    pub async fn update<E: DeserializeOwned, S: Serialize>(&self, id: impl Into<String>, body: S, options: Option<ViewOptions>) -> Result<E, ApiError> {
        // TODO: handle reauthentication if the update changes the password of the current user ?
        let url = format!("{}/api/collections/{}/records/{}{}", self.pb.base_url, self.collection_id_or_name, encode(&id.into()), options.unwrap_or_default().to_url_query());
        let headers = self.get_auth_headers();
        let body = self.pb.client
            .patch(&url)
            .headers(headers)
            .json(&body)
            .send().await?
            .text().await?;
        // If the update concerns the current user,
        // then the response is stored as the record of the auth store.
        let mut auth_store = self.pb.auth_store.lock().unwrap();
        if auth_store.is_some() {
            let auth_coll_id = auth_store.collection_id.clone().unwrap();
            let auth_coll_name = auth_store.collection_name.clone().unwrap();
            if self.collection_id_or_name == auth_coll_id || self.collection_id_or_name == auth_coll_name {
                let record = serde_json::from_str::<RecordIdOnly>(&body);
                if let Ok(record) = record {
                    let auth_rec_id = auth_store.record_id.clone().unwrap();
                    if record.id == auth_rec_id {
                        auth_store.set_record_id(record.id.clone());
                        let auth_record = serde_json::from_str::<T>(&body);
                        if let Ok(auth_record) = auth_record {
                            auth_store.set_record(auth_record);
                        }
                    }
                }
            }
        }
        self.handle_response_body(&body).await
    }

    /// Authenticates using an identity field (usually an email address) and a password.
    pub async fn auth_with_password(&mut self, identity: impl Into<String>, password: impl Into<String>) -> Result<AuthResponse<T>, ApiError> {
        let url = format!("{}/api/collections/{}/auth-with-password", self.pb.base_url, self.collection_id_or_name);
        let payload = AuthRequestPayload {
            password: password.into(),
            identity: identity.into(),
        };
        let body = self.pb.client.post(&url).header("Content-Type", "application/json").json(&payload).send().await?.text().await?;
        let tmp = self.handle_response_body::<AuthResponse<DefaultAuthResponseRecord>>(&body).await;
        let result = self.handle_response_body::<AuthResponse<T>>(&body).await;
        if let Ok(response) = &tmp {
            let token = response.token.clone();
            let mut lock = self.pb.auth_store.lock().unwrap();
            lock.set_token(token);
            lock.set_collection(response.record.collection_name.clone(), response.record.collection_id.clone());
            lock.set_record_id(response.record.id.clone());
            if let Ok(actual_result) = &result {
                lock.set_record(actual_result.record.clone());
            }
        }
        result
    }
}
