
use serde::*;
use once_cell::sync::Lazy;

#[derive(Deserialize, Debug)]
#[allow(unused_variables, dead_code)]
struct ExistingCollection {
    fields: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(unused_variables, dead_code)]
struct UsersRecord {
    created: String,
    updated: String,
    email: String,
    id: String,
    name: String,
    email_visibility: bool,
    verified: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(unused_variables, dead_code)]
struct ArticleRecord {
    created: String,
    updated: String,
    id: String,
    name: String,
    price: f64,
    public: bool,
}

#[derive(Deserialize, Debug)]
#[allow(unused_variables, dead_code)]
struct Data {
    users: Vec<UsersRecord>,
    articles: Vec<ArticleRecord>,
}

#[derive(Deserialize, Debug)]
#[allow(unused_variables, dead_code)]
struct ExistingCollections {
    users: ExistingCollection,
    articles: ExistingCollection,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(unused_variables, dead_code)]
struct TestData {
    existing_collections: ExistingCollections,
    data: Data,
}

static DEMO: Lazy<TestData> = Lazy::new(|| {
    let json = include_str!("../demo-data.json");
    serde_json::from_str(json).expect("invalid demo data JSON file")
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_demo_data() {
        assert!(!DEMO.existing_collections.articles.fields.is_empty());
        assert!(!DEMO.existing_collections.users.fields.is_empty());
        assert!(!DEMO.data.articles.is_empty());
        assert!(!DEMO.data.users.is_empty());
    }

    #[test]
    fn test_init_pocketbase() {
        let pb1 = PocketBase::default("http://localhost:8091/").expect("Could not create default PocketBase instance");
        let pb2: PocketBase<DefaultAuthRecord> = PocketBase::new("http://localhost:8091/").expect("Could not create new PocketBase instance");
        assert_eq!(pb1.base_url(), "http://localhost:8091");
        assert_eq!(pb2.base_url(), "http://localhost:8091");
    }

    #[test]
    fn test_empty_auth_store() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let auth_store = pb.auth_store();
        assert!(auth_store.token.is_none());
        assert!(auth_store.record.is_none());
        assert!(auth_store.collection_name.is_none());
        assert!(auth_store.collection_id.is_none());
    }

    #[tokio::test]
    async fn test_authless_get_one() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let id = "x4esjr8xe1yrrzv".to_string();
        let demo_record = DEMO.data.articles.iter().find(|x| { x.id == id }).expect("Missing demo article");
        let fetched_record = pb.collection("articles").get_one::<ArticleRecord>(&id, None).await.expect("Could not fetch article.");
        assert_eq!(fetched_record.id, id);
        assert_eq!(fetched_record.id, demo_record.id);
        assert_eq!(fetched_record.name, demo_record.name);
        assert_eq!(fetched_record.price, demo_record.price);
        assert_eq!(fetched_record.public, demo_record.public);
    }

    #[tokio::test]
    async fn test_authless_get_list() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let fetched_records = pb.collection("articles").get_list::<ArticleRecord>(&ListOptions::default()).await.expect("Could not fetch articles.");
        assert!(!fetched_records.items.is_empty());
        assert_eq!(fetched_records.total_pages, 1.);
        assert_eq!(fetched_records.total_items as usize, DEMO.data.articles.len());
        for (index, article) in fetched_records.items.iter().enumerate() {
            assert_eq!(article.id, DEMO.data.articles[index].id);
        }
    }

    #[tokio::test]
    async fn test_authless_get_list_with_filter() {
        let options = ListOptions {
            filter: Some("public=true".to_string()),
            ..ListOptions::default()
        };
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let demo_records = DEMO.data.articles.iter().filter(|x| { x.public }).collect::<Vec<&ArticleRecord>>();
        let fetched_records = pb.collection("articles").get_list::<ArticleRecord>(&options).await.expect("Could not fetch articles.");
        assert!(demo_records.len() > 0);
        assert!(fetched_records.items.len() > 0);
        assert_eq!(fetched_records.total_items as usize, demo_records.len());
        assert_eq!(fetched_records.total_pages, 1.);
        for (i, fetched_record) in fetched_records.items.iter().enumerate() {
            assert_eq!(fetched_record.id, demo_records[i].id);
            assert!(fetched_record.public);
        }
    }

    #[tokio::test]
    async fn test_authless_get_full_list() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let fetched_records = pb.collection("articles").get_full_list::<ArticleRecord>().await.expect("Could not fetch articles.");
        assert_eq!(fetched_records.len(), DEMO.data.articles.len());
        for (index, fetched_record) in fetched_records.iter().enumerate() {
            assert_eq!(fetched_record.id, DEMO.data.articles[index].id);
        }
    }
}