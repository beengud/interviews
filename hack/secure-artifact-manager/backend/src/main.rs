use actix_web::{web, App, HttpServer, HttpResponse, Responder, get, post, put, delete};
use actix_files::NamedFile;
use actix_cors::Cors;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use std::io::Write;
use uuid::Uuid;
use chrono::Utc;
use sha2::{Sha256, Digest};

// #[derive(Serialize, Deserialize, Clone)]
// struct Artifact {
//     id: String,
//     name: String,
//     original_filename: String,
//     metadata: String,
// }

#[derive(Serialize, Deserialize, Clone)]
pub struct ArtifactMetadata {
    pub id: String,
    pub original_filename: String,

    #[serde(default)]
    pub size: u64,

    #[serde(default)]
    pub sha256: String,

    #[serde(default)]
    pub upload_time: String,

    pub metadata: String,
}

#[get("/artifacts")]
async fn list_artifacts() -> impl Responder {
    let entries = fs::read_dir("./data").unwrap();
    let mut artifacts = vec![];

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let contents = std::fs::read_to_string(&path).unwrap();
            let meta: ArtifactMetadata = serde_json::from_str(&contents).unwrap();
            artifacts.push(meta);
        }
    }

    HttpResponse::Ok().json(artifacts)
}

#[post("/upload")]
async fn upload_artifact(mut payload: Multipart) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let data_path = format!("./data/{}", id);
    let meta_path = format!("./data/{}.json", id);

    let mut hasher = Sha256::new();
    let mut file = std::fs::File::create(&data_path).unwrap();
    let mut total_bytes = 0;

    let mut original_filename = "unknown".to_string();

    while let Some(field_result) = payload.next().await {
        if let Ok(mut field) = field_result {

            if let Some(fname) = field.content_disposition().get_filename() {
                original_filename = fname.to_string();
            }

            while let Some(chunk_result) = field.next().await {
                let data = chunk_result.unwrap();
                total_bytes += data.len() as u64;
                hasher.update(&data);
                file.write_all(&data).unwrap();
            }
        }
    }

    let sha256 = format!("{:x}", hasher.finalize());
    let upload_time = Utc::now().to_rfc3339();

    let meta = ArtifactMetadata {
        id: id.clone(),
        original_filename,
        size: total_bytes,
        sha256,
        upload_time,
        metadata: "Uploaded via UI".to_string(),
    };

    std::fs::write(&meta_path, serde_json::to_string(&meta).unwrap()).unwrap();

    log::info!("Uploaded file: {} (saved as: {})", meta.original_filename, id);
    HttpResponse::Ok().body(id)
}

#[get("/download/{id}")]
async fn download_artifact(path: web::Path<String>) -> actix_web::Result<NamedFile> {
    let file_path = format!("./data/{}", path.into_inner());
    log::info!("Downloading artifact: {}", file_path);
    Ok(NamedFile::open(file_path)?)
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App, http::StatusCode};
    use futures_util::stream::StreamExt as _;
    use std::{fs, io::Read};
    use std::str;
    use serde_json;

    #[actix_web::test]
    async fn test_list_artifacts_empty() {
        // Ensure data directory is fresh
        let _ = fs::remove_dir_all("./data");
        fs::create_dir_all("./data").unwrap();

        let app = test::init_service(
            App::new().service(list_artifacts)
        ).await;
        let req = test::TestRequest::get().uri("/artifacts").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<ArtifactMetadata> = test::read_body_json(resp).await;
        assert!(body.is_empty());
    }

    #[actix_web::test]
    async fn test_list_artifacts_with_meta() {
        let _ = fs::remove_dir_all("./data");
        fs::create_dir_all("./data").unwrap();

        let meta = ArtifactMetadata {
            id: "test-id".into(),
            original_filename: "file.txt".into(),
            size: 123,
            sha256: "abc".into(),
            upload_time: "2025-01-01T00:00:00Z".into(),
            metadata: "meta".into(),
        };
        fs::write("./data/test-id.json", serde_json::to_string(&meta).unwrap()).unwrap();

        let app = test::init_service(
            App::new().service(list_artifacts)
        ).await;
        let req = test::TestRequest::get().uri("/artifacts").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<ArtifactMetadata> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
        assert_eq!(body[0], meta);
    }

    #[actix_web::test]
    async fn test_upload_and_download_artifact() {
        // Reset data directory
        let _ = fs::remove_dir_all("./data");
        fs::create_dir_all("./data").unwrap();

        // Prepare multipart payload
        let boundary = "TESTBOUNDARY";
        let payload = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"hello.txt\"\r\n\
             \r\n\
             Hello, world!\r\n\
             --{boundary}--\r\n",
            boundary=boundary
        );
        let content_type = format!("multipart/form-data; boundary={}", boundary);

        let app = test::init_service(
            App::new()
                .service(upload_artifact)
                .service(download_artifact)
        ).await;

        // Upload
        let req = test::TestRequest::post()
            .uri("/upload")
            .insert_header(("Content-Type", content_type.clone()))
            .set_payload(payload.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = test::read_body(resp).await;
        let id = str::from_utf8(&body_bytes).unwrap();
        assert!(!id.is_empty());

        // Verify metadata file
        let meta_path = format!("./data/{}.json", id);
        let meta_contents = fs::read_to_string(&meta_path).unwrap();
        let meta: ArtifactMetadata = serde_json::from_str(&meta_contents).unwrap();
        assert_eq!(meta.id, id);
        assert_eq!(meta.original_filename, "hello.txt");
        assert_eq!(meta.size, "Hello, world!".len() as u64);

        // Download
        let req = test::TestRequest::get()
            .uri(&format!("/download/{}", id))
            .to_request();
        let mut resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let mut downloaded = Vec::new();
        while let Some(chunk) = resp.take_body().next().await {
            downloaded.extend_from_slice(&chunk.unwrap());
        }
        assert_eq!(downloaded, b"Hello, world!");
    }
}