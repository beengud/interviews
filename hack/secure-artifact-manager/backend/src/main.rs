use actix_web::{App, HttpServer};
use actix_cors::Cors;
// use std::path::PathBuf;
use backend::{
    list_artifacts,
    upload_artifact,
    download_artifact
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("./data")?;
    env_logger::init();

    log::info!("Starting backend on http://0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
        .wrap(Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600),
        )
        .service(list_artifacts)
        .service(upload_artifact)
        .service(download_artifact)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
