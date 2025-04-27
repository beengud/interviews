use crate::client::{api_base_url, http_client};
use colored::*;

pub async fn list() {
    let client = http_client();
    let url = format!("{}/artifacts", api_base_url());

    match client.get(&url).send().await {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(json) => {
                    println!("{}", "Artifacts:".green().bold());
                    if let Some(array) = json.as_array() {
                        for artifact in array {
                            let id = artifact.get("id").and_then(|v| v.as_str()).unwrap_or("<unknown>");
                            let name = artifact.get("filename").and_then(|v| v.as_str()).unwrap_or("<no name>");
                            println!("- {}: {}", id.cyan(), name.yellow());
                        }
                    } else {
                        println!("{}", "No artifacts found.".yellow());
                    }
                }
                Err(_) => eprintln!("{}", "Failed to parse JSON.".red()),
            }
        }
        Err(e) => eprintln!("{}: {:?}", "Request failed".red(), e),
    }
}