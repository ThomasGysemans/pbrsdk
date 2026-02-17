use serde::*;
use once_cell::sync::Lazy;

#[derive(Deserialize, Debug)]
#[allow(unused_variables, dead_code)]
struct ExistingCollection {
    fields: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(unused_variables, dead_code)]
struct UsersRecord {
    created: String,
    updated: String,
    email: String,
    id: String,
    name: String,
    password: String,
    password_confirm: String,
    email_visibility: bool,
    verified: bool,
}

#[derive(Serialize, Debug)]
struct UsersRecordPayload {
    name: String,
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Serialize, Debug)]
struct ArticleRecordPayload {
    // "id" is included just for testing purposes
    // obviously "id" isn't necessary in the payload
    // of the POST requests.
    id: String,
    name: String,
    price: f64,
    public: bool,
}

#[derive(Serialize, Debug)]
struct ArticleUpdatePayload {
    name: String,
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

#[allow(dead_code)]
static DEMO: Lazy<TestData> = Lazy::new(|| {
    let json = include_str!("../demo-data.json");
    serde_json::from_str(json).expect("invalid demo data JSON file")
});

#[cfg(test)]
mod tests {
    use serial_test::serial;
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
    #[serial]
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
    #[serial]
    async fn test_authless_get_list() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let fetched_records = pb.collection("articles").get_list::<ArticleRecord>(ListOptions::default()).await.expect("Could not fetch articles.");
        assert!(!fetched_records.items.is_empty());
        assert_eq!(fetched_records.total_pages, 1);
        assert_eq!(fetched_records.total_items as usize, DEMO.data.articles.len());
        for article in fetched_records.items.iter() {
            DEMO.data.articles.iter().find(|x| { x.id == article.id }).expect("Missing demo article");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_authless_get_list_with_filter() {
        let options = ListOptions {
            filter: Some("public=true".to_string()),
            ..ListOptions::default()
        };
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let demo_records = DEMO.data.articles.iter().filter(|x| { x.public }).collect::<Vec<&ArticleRecord>>();
        let fetched_records = pb.collection("articles").get_list::<ArticleRecord>(options).await.expect("Could not fetch articles.");
        assert!(demo_records.len() > 0);
        assert!(fetched_records.items.len() > 0);
        assert_eq!(fetched_records.items.len(), demo_records.len());
        assert_eq!(fetched_records.total_items as usize, demo_records.len());
        assert_eq!(fetched_records.total_pages, 1);
        for (i, fetched_record) in fetched_records.items.iter().enumerate() {
            assert_eq!(fetched_record.id, demo_records[i].id);
            assert!(fetched_record.public);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_authless_get_list_with_filter_and_skip_total() {
        let options = ListOptions {
            skip_total: Some(true),
            filter: Some("public=true".to_string()),
            ..ListOptions::default()
        };
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let demo_records = DEMO.data.articles.iter().filter(|x| { x.public }).collect::<Vec<&ArticleRecord>>();
        let fetched_records = pb.collection("articles").get_list::<ArticleRecord>(options).await.expect("Could not fetch articles.");
        assert!(demo_records.len() > 0);
        assert!(fetched_records.items.len() > 0);
        assert_eq!(fetched_records.items.len(), demo_records.len());
        assert_eq!(fetched_records.total_items, -1);
        assert_eq!(fetched_records.total_pages, -1);
        for (i, fetched_record) in fetched_records.items.iter().enumerate() {
            assert_eq!(fetched_record.id, demo_records[i].id);
            assert!(fetched_record.public);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_authless_get_list_paginated() {
        let options = ListOptions {
            page: Some(2),
            per_page: Some(2),
            filter: Some("public=true".to_string()),
            ..ListOptions::default()
        };
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let demo_records = DEMO.data.articles.iter().filter(|x| { x.public }).collect::<Vec<&ArticleRecord>>();
        let demo_page_2 = demo_records.chunks(2).nth(1).unwrap();
        let fetched_records = pb.collection("articles").get_list::<ArticleRecord>(options).await.expect("Could not fetch articles.");
        assert!(demo_records.len() > 2, "Demo records don't have enough 'public' articles to do this test correctly");
        assert!(demo_records.len() > 0);
        assert_eq!(fetched_records.page, 2);
        assert_eq!(fetched_records.per_page, 2);
        assert_eq!(fetched_records.total_items as usize, demo_records.len());
        assert_eq!(fetched_records.total_pages, demo_records.len().div_ceil(2) as i64);
        assert_eq!(demo_page_2.len(), fetched_records.items.len());
        for (i, fetched_record) in fetched_records.items.iter().enumerate() {
            assert_eq!(fetched_record.id, demo_page_2[i].id);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_authless_get_full_list() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let fetched_records = pb.collection("articles").get_full_list::<ArticleRecord>().await.expect("Could not fetch articles.");
        assert_eq!(fetched_records.len(), DEMO.data.articles.len());
        for article in fetched_records.iter() {
            DEMO.data.articles.iter().find(|x| { x.id == article.id }).expect("Missing demo article");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_authless_get_list_and_sort() {
        let options = ListOptions {
            sort: Some("+name".to_string()),
            ..ListOptions::default()
        };
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let mut sorted_demo = DEMO.data.articles.clone();
        sorted_demo.sort_by(|a, b| a.name.cmp(&b.name));
        let fetched_records = pb.collection("articles").get_list::<ArticleRecord>(options).await.expect("Could not fetch articles.");
        let sorted_names = sorted_demo.iter().map(|x| x.name.clone()).collect::<Vec<String>>();
        let original_names = DEMO.data.articles.iter().map(|x| x.name.clone()).collect::<Vec<String>>();
        assert!(fetched_records.items.len() > 0);
        assert_eq!(fetched_records.items.len(), DEMO.data.articles.len());
        assert_eq!(fetched_records.items.len(), sorted_demo.len());
        assert_ne!(sorted_names, original_names, "The articles of the demo are already sorted, meaning this test is not reliable.");
        for (i, fetched_record) in fetched_records.items.iter().enumerate() {
            assert_eq!(fetched_record.id, sorted_demo[i].id);
            assert_eq!(fetched_record.name, sorted_demo[i].name);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_authless_get_first_list_item() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let demo = DEMO.data.articles.iter().find(|x| { x.public }).expect("Could not find demo article with 'public' set to true.");
        let fetched_record = pb.collection("articles").get_first_list_item::<ArticleRecord>("public=true", None).await.expect("Could not fetch article.");
        assert_eq!(fetched_record.id, demo.id);
        assert_eq!(fetched_record.public, demo.public);
        assert!(fetched_record.public);
    }

    #[tokio::test]
    #[serial]
    async fn test_auth_simple_user() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let demo_user = &DEMO.data.users[0];
        let res = pb.collection("users").auth_with_password(&demo_user.email, &demo_user.password).await.expect("Could not authenticate user.");
        let auth_store = pb.auth_store();
        assert!(!res.token.is_empty());
        assert!(auth_store.token.is_some());
        assert!(auth_store.record.is_some());
        assert!(auth_store.collection_id.is_some());
        assert!(auth_store.collection_name.is_some());
        assert!(auth_store.record_id.is_some());
        assert_eq!(auth_store.collection_name.as_ref().unwrap(), "users");
        assert_eq!(auth_store.collection_name.as_ref().unwrap().to_string(), res.record.collection_name);
        assert_eq!(auth_store.collection_id.as_ref().unwrap().to_string(), res.record.collection_id);
        assert_eq!(auth_store.token.as_ref().unwrap().to_string(), res.token);
        assert_eq!(auth_store.record.as_ref().unwrap().id, res.record.id);
        assert_eq!(auth_store.record.as_ref().unwrap().id, demo_user.id);
        assert_eq!(auth_store.record.as_ref().unwrap().id, auth_store.record_id.as_ref().unwrap().to_string());
        assert_eq!(auth_store.record.as_ref().unwrap().name.as_ref().unwrap().to_string(), res.record.name.unwrap());
        assert_eq!(auth_store.record.as_ref().unwrap().name.as_ref().unwrap().to_string(), demo_user.name);
        assert!(auth_store.is_some());
        assert!(auth_store.is_valid());
        assert!(!auth_store.is_superuser());
    }

    #[tokio::test]
    #[serial]
    async fn test_auth_superuser() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let res = pb.collection("_superusers").auth_with_password("thomas@gysemans.dev", "thomasgysemans").await.expect("Could not authenticate super user.");
        let auth_store = pb.auth_store();
        assert!(!res.token.is_empty());
        assert!(auth_store.token.is_some());
        assert!(auth_store.record.is_some());
        assert!(auth_store.collection_id.is_some());
        assert!(auth_store.collection_name.is_some());
        assert!(auth_store.record_id.is_some());
        assert_eq!(auth_store.collection_name.as_ref().unwrap(), "_superusers");
        assert_eq!(auth_store.collection_name.as_ref().unwrap().to_string(), res.record.collection_name);
        assert_eq!(auth_store.collection_id.as_ref().unwrap().to_string(), res.record.collection_id);
        assert_eq!(auth_store.token.as_ref().unwrap().to_string(), res.token);
        assert_eq!(auth_store.record.as_ref().unwrap().id, res.record.id);
        assert_eq!(auth_store.record.as_ref().unwrap().id, auth_store.record_id.as_ref().unwrap().to_string());
        assert!(res.record.name.is_none());
        assert!(auth_store.record.as_ref().unwrap().name.is_none());
        assert!(auth_store.is_some());
        assert!(auth_store.is_valid());
        assert!(auth_store.is_superuser());
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_and_create() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let _ = pb.collection("_superusers").auth_with_password("thomas@gysemans.dev", "thomasgysemans").await;
        assert!(pb.auth_store().token.is_some());
        let demo = DEMO.data.articles[0].clone();
        assert!(pb.collection("articles").get_one::<ArticleRecord>(demo.id.clone(), None).await.is_ok(), "The demo JS script needs to be re-run because it's out of sync.");
        let _ = pb.collection("articles").delete(demo.id.clone()).await.expect("Could not delete article.");
        assert!(pb.collection("articles").get_one::<ArticleRecord>(demo.id.clone(), None).await.is_err());
        let payload = ArticleRecordPayload {
            id: demo.id.clone(),
            name: demo.name.clone(),
            price: demo.price,
            public: demo.public,
        };
        let created_record: ArticleRecord = pb.collection("articles").create(payload, None).await.expect("Could not create article.");
        let fetched_record: ArticleRecord = pb.collection("articles").get_one(demo.id.clone(), None).await.expect("Could not fetch article.");
        assert_eq!(created_record.id, fetched_record.id);
        assert_eq!(created_record.id, demo.id);
        assert_eq!(created_record.name, demo.name);
        assert_eq!(created_record.public, demo.public);
        assert_eq!(created_record.price, demo.price);
        assert_ne!(created_record.updated, demo.updated);
        assert_ne!(created_record.created, demo.created);
    }

    #[tokio::test]
    #[serial]
    async fn test_update() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let _ = pb.collection("_superusers").auth_with_password("thomas@gysemans.dev", "thomasgysemans").await;
        assert!(pb.auth_store().token.is_some());
        let demo = DEMO.data.articles[0].clone();
        let fetched_article = pb.collection("articles").get_one::<ArticleRecord>(demo.id.clone(), None).await.expect("Could not fetch test article");
        assert_eq!(fetched_article.id, demo.id);
        assert_eq!(fetched_article.name, demo.name);
        let updated_article: ArticleRecord = pb.collection("articles").update(demo.id.clone(), ArticleUpdatePayload { name: "new name".into() }, None).await.expect("Could not update article.");
        assert_eq!(demo.id, updated_article.id);
        assert_eq!(updated_article.name, "new name");
        assert_ne!(updated_article.name, demo.name);
        let new_update: ArticleRecord = pb.collection("articles").update(demo.id.clone(), ArticleUpdatePayload { name: demo.name.clone() }, None).await.expect("Could not update article again.");
        assert_eq!(demo.id, new_update.id);
        assert_eq!(new_update.name, demo.name);
        let fetched_again = pb.collection("articles").get_one::<ArticleRecord>(demo.id.clone(), None).await.unwrap();
        assert_eq!(fetched_again.id, demo.id);
        assert_eq!(fetched_again.name, demo.name);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_of_current_user() {
        let pb = PocketBase::default("http://localhost:8091/").unwrap();
        let demo_user = DEMO.data.users[0].clone();
        let _ = pb.collection("users").auth_with_password(demo_user.email, demo_user.password).await;
        assert!(pb.auth_store().token.is_some());
        assert_eq!(pb.auth_store().record_id.unwrap(), demo_user.id);
        assert_eq!(pb.auth_store().record.unwrap().name.unwrap(), demo_user.name);
        let updated_user_record: DefaultAuthRecord = pb.collection("users").update(demo_user.id.clone(), UsersRecordPayload { name: "John Doe".to_string() }, None).await.expect("Could not update user record.");
        assert_eq!(pb.auth_store().record_id.unwrap(), demo_user.id);
        assert_eq!(updated_user_record.name.as_ref().unwrap().to_string(), "John Doe".to_string());
        assert_eq!(pb.auth_store().record.unwrap().name.unwrap(), updated_user_record.name.as_ref().unwrap().to_string());
        assert_ne!(pb.auth_store().record.unwrap().name.unwrap(), demo_user.name.clone());
        let recover_update: DefaultAuthRecord = pb.collection("users").update(demo_user.id.clone(), UsersRecordPayload { name: demo_user.name.clone() }, None).await.expect("Could not update back user record.");
        assert_eq!(pb.auth_store().record_id.unwrap(), demo_user.id);
        assert_eq!(pb.auth_store().record.unwrap().name.unwrap(), demo_user.name.clone());
        assert_eq!(pb.auth_store().record.unwrap().name.unwrap(), recover_update.name.unwrap());
    }
}