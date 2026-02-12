use std::sync::Arc;
use serde::de::DeserializeOwned;
use crate::error::ApiError;
use crate::pocketbase::PocketBaseRef;

/// Handles requests meant to concern the collections themselves,
/// rather than the records they contain.
pub struct CollectionService<T>
where T: DeserializeOwned + Clone {
    pub(crate) base_crud_path: &'static str,
    pub(crate) pb: Arc<PocketBaseRef<T>>,
}

impl<T> CollectionService<T>
where T: DeserializeOwned + Clone {
    /// All API requests concerning the collections themselves
    /// go through this path. It's a constant, it must not change.
    pub fn base_crud_path(&self) -> &'static str {
        self.base_crud_path
    }

    /// Gets the full list of collections.
    pub async fn get_full_list(&self) -> Result<String, ApiError> {
        // TODO: requires authentication header
        let url = format!("{}{}", self.pb.base_url, self.base_crud_path);
        let body = self.pb.client.get(url).send().await?.text().await?;
        Ok(body)
    }
}
