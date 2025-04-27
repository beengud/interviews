use reqwest::multipart::{Form, Part};
use reqwest::Client;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use colored::*;

pub async fn upload(path: &str) {
    let file_path = Path::new(path);
    if !file_path.exists() {
        eprintln!("{}: {}", "File does not exist".red(), path);
        return;
    }

    // Read file into memory
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    let file_part = Part::bytes(buffer)
        .file_name(
            file_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        )
        .mime_str("application/octet-stream")
        .unwrap();

    let form = Form::new().part("artifact", file_part);

    let client = Client::new();
    let url = format!("{}/artifacts/upload", crate::client::api_base_url());
    match client.post(url).multipart(form).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("{}", "Upload successful.".green());
            } else {
                eprintln!("{}", format!("Upload failed: {}", resp.status()).red());
            }
        }
        Err(e) => {
            eprintln!("{}: {:?}", "Request failed".red(), e);
        }
    }
}