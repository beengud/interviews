fn main() {
    println!("cargo:rerun-if-changed=src/"); // rerun build if src/ changes
    println!("cargo:rerun-if-changed=migrations/"); // if you later use sqlx migrations

    if std::env::var("CARGO_FEATURE_OFFLINE").is_err() {
        if let Err(e) = std::process::Command::new("cargo")
            .args(&["sqlx", "prepare", "--check"])
            .status()
        {
            panic!("Failed to run `cargo sqlx prepare --check`: {}", e);
        }
    }
}