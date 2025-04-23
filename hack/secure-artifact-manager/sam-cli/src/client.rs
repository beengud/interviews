use reqwest::Client;

pub fn api_base_url() -> String {
    std::env::var("SAM_API_URL").unwrap_or_else(|_| "http://localhost:8000/api".into())
}

pub fn http_client() -> Client {
    Client::new()
}