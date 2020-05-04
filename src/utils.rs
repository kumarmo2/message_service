use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

pub fn create_client() -> Client {
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kumarmo2"));
    Client::builder()
        .default_headers(headers)
        .build()
        .expect("could not create http client")
}
