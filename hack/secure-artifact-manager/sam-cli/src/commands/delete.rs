use reqwest::Client;
use colored::*;

pub async fn delete(id: &str) {
    let client = Client::new();
    let url = format!("http://localhost:8000/api/artifacts/{}", id);
    match client.delete(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("{} {}", "Deleted artifact".green(), id.cyan());
            } else {
                eprintln!("{}", format!("Delete failed: {}", resp.status()).red());
            }
        }
        Err(e) => eprintln!("{}: {:?}", "Request failed".red(), e),
    }
}