#![allow(unused)]

use serde::Deserialize;
use pbrsdk::*;

#[base_system_fields]
#[derive(Debug)]
struct Article {
    name: String,
    price: f64,
    public: bool,
}

#[base_system_fields]
#[derive(Debug, Clone)]
struct CustomUserType {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    // to use a custom user type then call PocketBase::<CustomUserType>::new();
    let pb = PocketBase::default("http://localhost:8091/").unwrap();
    authenticate(&pb).await;
    let articles = fetch_articles(&pb).await;
    println!("articles: {:#?}", articles);
}

async fn authenticate(pb: &PocketBase) {
    let response = pb.collection("_superusers").auth_with_password("thomas@gysemans.dev", "thomasgysemans").await;
    if let Err(err) = response {
        if let ApiError::Http(_, _) = err {
            panic!("{}", err);
        }
    } else {
        let auth = pb.auth_store();
        println!("Authenticated as {:#?}", auth.record.as_ref().unwrap());
        println!("is valid : {:#?}", auth.is_valid());
        println!("is super user : {:#?}", auth.is_superuser());
    }
}

async fn fetch_articles(pb: &PocketBase) -> Vec<Article> {
    let response = pb.collection("articles").get_full_list().await;
    if let Err(err) = response {
        panic!("{}", err);
    } else {
        response.unwrap().items
    }
}