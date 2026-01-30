use serde::Deserialize;
use pbrsdk::*;

#[base_system_fields]
#[derive(Debug, Deserialize)]
struct Article {
    name: String,
    price: f64,
    public: bool,
}

#[base_system_fields]
#[derive(Debug, Deserialize, Clone)]
struct CustomUserType {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    // to use a custom user type then call PocketBase::<CustomUserType>::new();
    let pb = PocketBase::default("http://localhost:8091/").unwrap();
    let response = pb.collection("users").auth_with_password("thomas@gysemans.dev", "qwertyui").await;
    if let Err(err) = response {
        if let ApiError::Http(_, _) = err {
            eprintln!("{}", err);
        }
    } else {
        let lock = pb.auth_store().lock();
        let r = lock.as_ref().unwrap().record().unwrap();
        println!("{:#?}", r);
    }
}
