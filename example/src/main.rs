use serde::Deserialize;
use pbrsdk::*;

#[base_system_fields]
#[derive(Debug, Deserialize)]
struct Article {
    name: String,
    price: f64,
    public: bool,
}

#[tokio::main]
async fn main() {
    let pb = PocketBase::new("http://localhost:8091/").unwrap();
    let response = pb.collection("Articles").get_full_list::<Article>().await;
    if let Err(err) = response {
        if let ApiError::Http(_, _) = err {
            eprintln!("{}", err);
        }
    } else {
        println!("{:#?}", response.unwrap().items);
    }
}
