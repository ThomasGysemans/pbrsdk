mod pocketbase;
mod error;
mod auth;

pub use pbrsdk_macros::base_system_fields;
pub use pocketbase::PocketBase;
pub use error::ApiError;
pub use auth::AuthStore;
pub use auth::DefaultAuthRecord;
