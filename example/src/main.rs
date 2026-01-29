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
    let result = pb.collection("Articles").unwrap().get_full_list::<Article>().await.unwrap();
    println!("{:#?}", result);
}
