use reqwest::Client;
use colored::*;

pub async fn scan(id: &str) {
    let client = Client::new();
    let url = format!("http://localhost:8000/api/artifacts/{}/scan", id);
    match client.post(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        println!("{}", "Scan results:".green().bold());
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    }
                    Err(_) => eprintln!("{}", "Failed to parse scan response.".red()),
                }
            } else {
                eprintln!("{}", format!("Scan failed: {}", resp.status()).red());
            }
        }
        Err(e) => eprintln!("{}: {:?}", "Request failed".red(), e),
    }
}