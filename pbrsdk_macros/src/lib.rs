use quote::{quote, ToTokens};
use proc_macro::TokenStream;
use syn::{parse_macro_input, Attribute, Field, Fields, ItemStruct};

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