#![warn(missing_docs)]

//! This crate is meant to provide a client SDK for PocketBase.
//! Any PocketBase server can be accessed with simple HTTP requests,
//! but this crate handles them for you, takes care of the authentication,
//! and gives you documentation directly in the code itself.
//!
//! If this crate is lacking a feature, you can simply make your own HTTP requests
//! and use the [AuthStore] instance to access the bearer token.
//!
//! ```rust
//! #[tokio::main]
//!  async fn main() {
//!     // to use a custom user type then call PocketBase::<CustomUserType>::new();
//!     let pb = PocketBase::default("http://localhost:8091/").unwrap();
//!     let response = pb.collection("_superusers").auth_with_password("email", "password").await;
//!     if let Err(err) = response {
//!         if let ApiError::Http(_, _) = err {
//!             eprintln!("{}", err);
//!         }
//!     } else {
//!         let auth = pb.auth_store();
//!         println!("is valid : {:#?}", auth.is_valid());
//!         println!("is super user : {:#?}", auth.is_superuser());
//!     }
//! }
//! ```

mod pocketbase;
mod error;
mod auth;
mod tests;
mod record_service;
mod collection_service;
mod common;

pub use pbrsdk_macros::base_system_fields;
pub use pocketbase::*;
pub use error::*;
pub use common::*;
pub use auth::*;
pub use record_service::*;
pub use collection_service::*;
