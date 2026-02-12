use serde::Deserialize;
use urlencoding::encode;

/// Describes an expected error returned by an API route of PocketBase.
#[derive(Debug, Deserialize)]
pub struct ResponseError {
    /// The message that the server returned.
    pub message: String,
    /// The HTTP status of the response.
    pub status: u16,
}

/// Options to view a collection's record.
#[derive(Debug, Default)]
pub struct ViewOptions {
    /// Comma separated string of the fields to return
    /// in the JSON response (by default returns all fields).
    pub fields: Option<String>,
    /// Auto expand record relations.
    pub expand: Option<String>,
    /// Specify the records order attribute.
    pub sort: Option<String>,
}

/// The query parameters for the API route `/api/collections/NAME/records`.
/// This route would return paginated results by default.
#[derive(Debug, Default, Clone)]
pub struct ListOptions {
    /// The page number.
    /// Starts at 1.
    pub page: Option<u64>,
    /// The number of items per page.
    pub per_page: Option<u64>,
    /// By default, the API returns the total number of items.
    /// If the targeted collection is huge, then skipping the total
    /// will avoid time-consuming computations.
    pub skip_total: Option<bool>,
    /// Filter the returned records.
    pub filter: Option<String>,
    /// Comma separated string of the fields to return
    /// in the JSON response (by default returns all fields).
    pub fields: Option<String>,
    /// Auto expand record relations.
    pub expand: Option<String>,
    /// Specify the records order attribute.
    pub sort: Option<String>,
}

impl ListOptions {
    /// Creates a simple instance that will only care about
    /// the page number and the amount of items per page.
    pub fn paginated(page: u64, per_page: u64) -> Self {
        ListOptions {
            page: Some(page),
            per_page: Some(per_page),
            ..ListOptions::default()
        }
    }

    /// Creates a simple instance that will only care about
    /// the page number and the amount of items per page,
    /// and also set "skip_total" to true.
    pub fn paginated_and_skip(page: u64, per_page: u64) -> Self {
        ListOptions {
            page: Some(page),
            per_page: Some(per_page),
            skip_total: Some(true),
            ..ListOptions::default()
        }
    }

    /// Build list options based on view options and filter.
    pub fn from_view(page: Option<u64>, per_page: Option<u64>, filter: Option<String>, view_options: Option<ViewOptions>) -> Self {
        ListOptions {
            page,
            per_page,
            filter,
            fields: if view_options.as_ref().is_some() { view_options.as_ref().unwrap().fields.clone() } else { None },
            expand: if view_options.as_ref().is_some() { view_options.as_ref().unwrap().expand.clone() } else { None },
            sort: if view_options.as_ref().is_some() { view_options.as_ref().unwrap().sort.clone() } else { None },
            skip_total: Some(true),
        }
    }

    pub(crate) fn to_url_query(&self) -> String {
        let mut url = "?".to_string();
        if let Some(page) = self.page { url.push_str(&format!("page={}&", page)); }
        if let Some(per_page) = self.per_page { url.push_str(&format!("perPage={}&", per_page)); }
        if let Some(skip_total) = self.skip_total { url.push_str(&format!("skipTotal={}&", if skip_total { "1" } else { "0" })); }
        if let Some(filter) = &self.filter { url.push_str(&format!("filter={}&", encode(filter).into_owned())); }
        if let Some(fields) = &self.fields { url.push_str(&format!("fields={}&", encode(fields).into_owned())); }
        if let Some(expand) = &self.expand { url.push_str(&format!("expand={}&", encode(expand).into_owned())); }
        if let Some(sort) = &self.sort { url.push_str(&format!("sort={}&", encode(sort).into_owned())); }
        if url.len() == 1 {
            return String::new();
        }
        url.strip_suffix("&").unwrap().to_string()
    }
}

impl ViewOptions {
    pub(crate) fn to_url_query(&self) -> String {
        let mut url = "?".to_string();
        if let Some(expand) = &self.expand { url.push_str(&format!("expand={}&", encode(expand).into_owned())); }
        if let Some(sort) = &self.sort { url.push_str(&format!("sort={}&", encode(sort).into_owned())); }
        if url.len() == 1 {
            return String::new();
        }
        url.strip_suffix("&").unwrap().to_string()
    }
}
