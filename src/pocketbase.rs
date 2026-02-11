use std::sync::{Arc, Mutex};
use reqwest::{Client, StatusCode};
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize};
use urlencoding::encode;
use crate::auth::{AuthRequest, AuthResponse, AuthStore, DefaultAuthRecord, DefaultAuthResponseRecord};
use crate::error::ApiError;

#[derive(Clone)]
pub struct CollectionService {
    base_crud_path: &'static str,
    client: Client,
    base_url: String,
}

pub struct Collection<T>
where T: DeserializeOwned + Clone {
    client: Client,
    auth_store: Arc<Mutex<AuthStore<T>>>,
    base_url: String,
    collection_id_or_name: String,
}

/// Creates a pocketbase instance from which requests to the server can be made.
/// It will also store essential pieces of information relative to the authentication.
pub struct PocketBase<T = DefaultAuthRecord>
where T: DeserializeOwned + Clone {
    auth_store: Arc<Mutex<AuthStore<T>>>,
    collections: CollectionService,
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub per_page: u64,
    pub total_items: i64,
    pub total_pages: i64,
}

#[derive(Debug, Deserialize)]
pub struct ResponseError {
    pub message: String,
    pub status: u16,
}

/// The query parameters for the API route `/api/collections/NAME/records`.
/// This route would return paginated results by default.
#[derive(Debug, Default, Clone)]
pub struct ListOptions {
    /// The page number.
    /// Starts at 1.
    pub page: Option<u64>,
    /// The number of items per page.
    pub per_page: Option<u64>,
    /// By default, the API returns the total number of items.
    /// If the targeted collection is huge, then skipping the total
    /// will avoid time-consuming computations.
    pub skip_total: Option<bool>,
    /// Filter the returned records.
    pub filter: Option<String>,
    /// Comma separated string of the fields to return
    /// in the JSON response (by default returns all fields).
    pub fields: Option<String>,
    /// Auto expand record relations.
    pub expand: Option<String>,
    /// Specify the records order attribute.
    pub sort: Option<String>,
}

/// Options to view a collection's record.
#[derive(Debug, Default)]
pub struct ViewOptions {
    /// Comma separated string of the fields to return
    /// in the JSON response (by default returns all fields).
    pub fields: Option<String>,
    /// Auto expand record relations.
    pub expand: Option<String>,
    /// Specify the records order attribute.
    pub sort: Option<String>,
}

impl ListOptions {
    /// Creates a simple instance that will only care about
    /// the page number and the amount of items per page.
    pub fn paginated(page: u64, per_page: u64) -> Self {
        ListOptions {
            page: Some(page),
            per_page: Some(per_page),
            ..ListOptions::default()
        }
    }

    /// Creates a simple instance that will only care about
    /// the page number and the amount of items per page,
    /// and also set "skip_total" to true.
    pub fn paginated_and_skip(page: u64, per_page: u64) -> Self {
        ListOptions {
            page: Some(page),
            per_page: Some(per_page),
            skip_total: Some(true),
            ..ListOptions::default()
        }
    }

    pub fn from_view(page: Option<u64>, per_page: Option<u64>, filter: Option<String>, view_options: Option<ViewOptions>) -> Self {
        ListOptions {
            page,
            per_page,
            filter,
            fields: if view_options.as_ref().is_none() { view_options.as_ref().unwrap().fields.clone() } else { None },
            expand: if view_options.as_ref().is_none() { view_options.as_ref().unwrap().expand.clone() } else { None },
            sort: if view_options.as_ref().is_none() { view_options.as_ref().unwrap().sort.clone() } else { None },
            skip_total: Some(true),
        }
    }

    pub(crate) fn to_url_query(&self) -> String {
        let mut url = "?".to_string();
        if let Some(page) = self.page { url.push_str(&format!("page={}&", page)); }
        if let Some(per_page) = self.per_page { url.push_str(&format!("perPage={}&", per_page)); }
        if let Some(skip_total) = self.skip_total { url.push_str(&format!("skipTotal={}&", if skip_total { "1" } else { "0" })); }
        if let Some(filter) = &self.filter { url.push_str(&format!("filter={}&", encode(filter).into_owned())); }
        if let Some(fields) = &self.fields { url.push_str(&format!("fields={}&", encode(fields).into_owned())); }
        if let Some(expand) = &self.expand { url.push_str(&format!("expand={}&", encode(expand).into_owned())); }
        if let Some(sort) = &self.sort { url.push_str(&format!("sort={}&", encode(sort).into_owned())); }
        if url.len() == 1 {
            return String::new();
        }
        url.strip_suffix("&").unwrap().to_string()
    }
}

impl ViewOptions {
    pub(crate) fn to_url_query(&self) -> String {
        let mut url = "?".to_string();
        if let Some(expand) = &self.expand { url.push_str(&format!("expand={}&", encode(expand).into_owned())); }
        if let Some(sort) = &self.sort { url.push_str(&format!("sort={}&", encode(sort).into_owned())); }
        if url.len() == 1 {
            return String::new();
        }
        url.strip_suffix("&").unwrap().to_string()
    }
}

impl<T> PocketBase<T>
where T: DeserializeOwned + Clone {
    /// Returns a reference to the base URL String that was given
    /// when initiating the [PocketBase] instance.
    pub fn base_url(&self) -> &String { &self.base_url }

    /// Returns a reference to the [CollectionService] instance.
    pub fn collections(&self) -> &CollectionService { &self.collections }

    /// Returns a clone of the AuthStore instance stored in the [PocketBase] struct.
    pub fn auth_store(&self) -> AuthStore<T> { self.auth_store.lock().unwrap().clone() }

    /// Creates a new instance of [PocketBase].
    pub fn new(base_url: impl Into<String>) -> Result<Self, ApiError> {
        let client = Client::new();
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

    /// Creates an instance of collection that you will later be able to fetch.
    /// In itself it doesn't check if the collection exists.
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
    /// Creates a [PocketBase] instance that will use the type [DefaultAuthRecord]
    /// for the authentication record. Use [PocketBase::new] to customize this type.
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

    fn get_auth_headers(&self) -> HeaderMap {
        let store = self.auth_store.lock();
        let token = store.as_ref().unwrap().token.clone();
        let mut headers: HeaderMap = HeaderMap::new();
        if let Some(token) = token {
            headers.insert("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        headers
    }

    /// Fetches pages of records.
    pub async fn get_list<E: DeserializeOwned>(&self, options: ListOptions) -> Result<ListResponse<E>, ApiError> {
        let url = format!("{}/api/collections/{}/records{}", self.base_url, self.collection_id_or_name, options.to_url_query());
        let headers = self.get_auth_headers();
        let body = self.client
            .get(&url)
            .headers(headers)
            .send().await?
            .text().await?;
        self.handle_response_body(&body).await
    }

    /// Fetches one record based on its ID, which must exist.
    /// If the ID isn't found, the server will return a 404 error.
    pub async fn get_one<E: DeserializeOwned>(&self, id: impl Into<String>, options: Option<ViewOptions>) -> Result<E, ApiError> {
        let url = format!("{}/api/collections/{}/records/{}{}", self.base_url, self.collection_id_or_name, encode(&id.into()), options.unwrap_or_default().to_url_query());
        let headers = self.get_auth_headers();
        let body = self.client
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
    /// For consistency with `get_one()`, this method will throw a 404 if the item wasn't found.
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

    /// Authenticates using an identity field (usually an email address) and a password.
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
            lock.set_collection(response.record.collection_name.clone(), response.record.collection_id.clone());
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