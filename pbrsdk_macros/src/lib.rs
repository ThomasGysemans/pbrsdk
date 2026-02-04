//! This crate only contains a single procedural macro aimed at
//! adding automatically 3 properties to a struct. Those properties are :
//!
//! - `collection_id`
//! - `collection_name`
//! - `id`
//!
//! Those properties are included in most responses of PocketBase.

use quote::{quote, ToTokens};
use proc_macro::TokenStream;
use syn::{parse_macro_input, Attribute, Field, Fields, ItemStruct};

/// Most requests to the PocketBase server will include 3 common properties in their response :
///
/// - `collectionId`
/// - `collectionName`
/// - `id`
///
/// Of course those properties must follow Rust's naming conventions.
/// Therefore, using serde, they are automatically renamed into:
///
/// - `collection_id`
/// - `collection_name`
/// - `id`
///
/// Those properties might come in handy sometimes.
/// Since you must define a struct for each response,
/// you can easily add those common properties using this
/// procedural macro.
///
/// Note that this macro also adds another attribute: `#[serde(rename_all = "camelCase")]`,
/// but `#[derive(Deserialize)]` is not added automatically, and yet is important.
///
/// # Example
///
/// ```
/// use pbrsdk_macros::base_system_fields;
/// use serde::*;
///
/// #[base_system_fields]
/// #[derive(Deserialize)]
/// struct ArticleRecord {
///     name: String,
///     price: f64,
/// }
/// # fn get_article() -> ArticleRecord {
/// #     ArticleRecord {
/// #         name: "".to_string(),
/// #         price: 0.0,
/// #         collection_id: "".to_string(),
/// #         collection_name: "articles".to_string(),
/// #         id: "".to_string(),
/// #     }
/// # }
/// fn main() {
///     let article = get_article();
///     println!("{}", article.collection_name);
/// }
/// ```
#[proc_macro_attribute]
pub fn base_system_fields(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    // Ensure named fields
    let fields = match &mut input.fields {
        Fields::Named(fields) => &mut fields.named,
        _ => panic!("#[base_system_fields] only supports structs with named fields"),
    };

    // Fields to inject
    let injected_fields: Vec<Field> = vec![
        syn::parse_quote!(pub id: String),
        syn::parse_quote!(pub collection_id: String),
        syn::parse_quote!(pub collection_name: String),
    ];

    // Prepend injected fields
    for field in injected_fields.into_iter().rev() {
        fields.insert(0, field);
    }

    // Inject #[serde(rename_all = "camelCase")] if it's not already present
    let already_has_serde = input.attrs.iter().any(|attr| {
        attr.path().is_ident("serde") &&
            attr.to_token_stream().to_string().contains("rename_all")
    });

    if !already_has_serde {
        let serde_attr: Attribute = syn::parse_quote!(#[serde(rename_all = "camelCase")]);
        input.attrs.push(serde_attr);
    }

    TokenStream::from(quote! {
        #input
    })
}