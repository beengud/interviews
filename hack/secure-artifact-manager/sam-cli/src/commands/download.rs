use reqwest::Client;
use std::fs::File;
use std::io::copy;
use colored::*;

pub async fn download(id: &str, output: &str) {
    let client = Client::new();
    let url = format!("http://localhost:8000/api/artifacts/{}/download", id);
    match client.get(&url).send().await {
        Ok(mut resp) => {
            if resp.status().is_success() {
                let mut file = File::create(output).expect("Could not create file");
                let mut content = resp.bytes().await.expect("Could not read bytes");
                copy(&mut content.as_ref(), &mut file).expect("Could not write to file");
                println!(
                    "{} {} → {}",
                    "Downloaded".green(),
                    id.cyan(),
                    output.yellow()
                );
            } else {
                eprintln!("{}", format!("Download failed: {}", resp.status()).red());
            }
        }
        Err(e) => eprintln!("{}: {:?}", "Request failed".red(), e),
    }
}