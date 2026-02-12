use std::sync::{Arc, Mutex};
use reqwest::{Client};
use serde::de::DeserializeOwned;
use crate::auth::{AuthStore, DefaultAuthRecord};
use crate::collection_service::CollectionService;
use crate::record_service::RecordService;
use crate::error::ApiError;

/// Creates a pocketbase instance from which requests to the server can be made.
/// It will also store essential pieces of information relative to the authentication.
pub struct PocketBase<T = DefaultAuthRecord>
where T: DeserializeOwned + Clone {
    inner: Arc<PocketBaseRef<T>>,
}

pub(crate) struct PocketBaseRef<T = DefaultAuthRecord>
where T: DeserializeOwned + Clone {
    pub(crate) auth_store: Arc<Mutex<AuthStore<T>>>,
    pub(crate) collections: CollectionService,
    pub(crate) client: Client,
    pub(crate) base_url: String,
}

impl<T> PocketBase<T>
where T: DeserializeOwned + Clone {
    /// Returns a reference to the base URL String that was given
    /// when initiating the [PocketBase] instance.
    pub fn base_url(&self) -> &String { &self.inner.base_url }

    /// Returns a reference to the [CollectionService] instance.
    pub fn collections(&self) -> &CollectionService { &self.inner.collections }

    /// Returns a clone of the AuthStore instance stored in the [PocketBase] struct.
    pub fn auth_store(&self) -> AuthStore<T> { self.inner.auth_store.lock().unwrap().clone() }

    /// Creates a new instance of [PocketBase].
    pub fn new(base_url: impl Into<String>) -> Result<Self, ApiError> {
        let client = Client::new();
        let url = base_url.into().strip_suffix("/").unwrap().to_owned();
        Ok(Self {
            inner: Arc::new(PocketBaseRef {
                client: client.clone(),
                base_url: url.clone(),
                auth_store: Arc::new(Mutex::new(AuthStore::default())),
                collections: CollectionService {
                    base_crud_path: "/api/collections",
                    base_url: url.clone(),
                    client: client.clone(),
                }
            })
        })
    }

    /// Creates an instance of collection that you will later be able to fetch.
    /// In itself it doesn't check if the collection exists.
    pub fn collection(&self, name_or_id: impl Into<String>) -> RecordService<T> {
        RecordService {
            collection_id_or_name: name_or_id.into(),
            pb: self.inner.clone(),
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