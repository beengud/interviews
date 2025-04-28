// mod lib;

use actix_web::{web, HttpResponse, Responder, get, post};
// use actix_files::NamedFile;
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
// use std::io::Write;
use uuid::Uuid;
use chrono::Utc;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use sqlx::Row;
use sqlx::FromRow;
use aws_sdk_s3::{Client as S3Client };
use aws_sdk_s3::primitives::ByteStream;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct ArtifactMetadata {
    pub id: String,
    pub original_filename: String,

    #[serde(default)]
    pub size_bytes: i64,

    #[serde(default)]
    pub sha256: String,

    #[serde(default)]
    pub upload_time: String,

    pub metadata: String,
}

#[get("/artifacts")]
pub async fn list_artifacts(db_pool: web::Data<PgPool>) -> impl Responder {
    // Query artifact metadata from PostgreSQL database
    let artifacts = sqlx::query_as::<_, ArtifactMetadata>(
        r#"
        SELECT id, original_filename, size_bytes, sha256, upload_time, metadata
        FROM artifacts
        ORDER BY upload_time DESC
        "#
    )
    .fetch_all(db_pool.get_ref())
    .await;

    match artifacts {
        Ok(arts) => HttpResponse::Ok().json(arts),
        Err(e) => {
            log::error!("Failed to fetch artifacts from DB: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch artifacts")
        }
    }
}

#[post("/upload")]
pub async fn upload_artifact(
    mut payload: Multipart,
    s3_client: web::Data<S3Client>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let bucket_name = "artifacts"; // Assume bucket exists

    let mut hasher = Sha256::new();
    let mut total_bytes = 0u64;

    let mut original_filename = "unknown".to_string();

    // Collect chunks in a buffer before uploading to S3
    let mut file_bytes = Vec::new();

    while let Some(field_result) = payload.next().await {
        if let Ok(mut field) = field_result {

            if let Some(fname) = field.content_disposition().get_filename() {
                original_filename = fname.to_string();
            }

            while let Some(chunk_result) = field.next().await {
                let data = chunk_result.unwrap();
                total_bytes += data.len() as u64;
                hasher.update(&data);
                file_bytes.extend_from_slice(&data);
            }
        }
    }

    // Upload to MinIO (S3)
    let byte_stream = ByteStream::from(file_bytes.clone());
    let put_result = s3_client.put_object()
        .bucket(bucket_name)
        .key(&id)
        .body(byte_stream)
        .send()
        .await;

    if let Err(e) = put_result {
        log::error!("Failed to upload artifact to MinIO: {}", e);
        return HttpResponse::InternalServerError().body("Failed to upload artifact");
    }

    let sha256 = format!("{:x}", hasher.finalize());
    let upload_time = Utc::now().to_rfc3339();

    let meta = ArtifactMetadata {
        id: id.clone(),
        original_filename: original_filename.clone(),
        size_bytes: total_bytes as i64,
        sha256,
        upload_time: upload_time.clone(),
        metadata: "Uploaded via UI".to_string(),
    };

    // Insert metadata into PostgreSQL database
    let insert_result = sqlx::query(
        r#"
        INSERT INTO artifacts (id, original_filename, size_bytes, sha256, upload_time, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#)
        .bind(&meta.id)
        .bind(&meta.original_filename)
        .bind(meta.size_bytes as i64)
        .bind(&meta.sha256)
        .bind(&meta.upload_time)
        .bind(&meta.metadata)
    .execute(db_pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        log::error!("Failed to insert artifact metadata into DB: {}", e);
        return HttpResponse::InternalServerError().body("Failed to save artifact metadata");
    }

    log::info!("Uploaded file: {} (saved as: {})", original_filename, id);
    HttpResponse::Ok().body(id)
}

#[get("/download/{id}")]
pub async fn download_artifact(
    path: web::Path<String>,
    s3_client: web::Data<S3Client>,
    db_pool: web::Data<PgPool>,
) -> actix_web::Result<HttpResponse> {
    let id = path.into_inner();
    let bucket_name = "artifacts"; // Assume bucket exists

    let row = sqlx::query(
        r#"
        SELECT original_filename
        FROM artifacts
        WHERE id = $1
        "#
    )
    .bind(&id)
    .fetch_one(db_pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("DB error fetching artifact metadata: {}", e);
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    let original_filename: String = row.try_get("original_filename").map_err(|e| {
        log::error!("DB error extracting original_filename: {}", e);
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    // Get object from S3 (MinIO)
    let get_obj_output = s3_client.get_object()
        .bucket(bucket_name)
        .key(id)
        .send()
        .await;

    let get_obj = match get_obj_output {
        Ok(obj) => obj,
        Err(e) => {
            log::error!("Failed to get artifact from MinIO: {}", e);
            return Err(actix_web::error::ErrorNotFound("Artifact not found"));
        }
    };

    // Collect the body bytes into memory
    let data = get_obj.body.collect().await.map_err(|e| {
        log::error!("Failed to read artifact body: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to read artifact body")
    })?;

    let content_type = get_obj.content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", original_filename)))
        .body(data.into_bytes()))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_artifacts)
        .service(upload_artifact)
        .service(download_artifact);
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use actix_web::{test, App, http::StatusCode};
//     use futures_util::stream::StreamExt as _;
//     use std::{fs, io::Read};
//     use std::str;
//     use serde_json;

//     #[actix_web::test]
//     async fn test_list_artifacts_empty() {
//         // Ensure data directory is fresh
//         let _ = fs::remove_dir_all("./data");
//         fs::create_dir_all("./data").unwrap();

//         let app = test::init_service(
//             App::new().service(list_artifacts)
//         ).await;
//         let req = test::TestRequest::get().uri("/artifacts").to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(resp.status(), StatusCode::OK);

//         let body: Vec<ArtifactMetadata> = test::read_body_json(resp).await;
//         assert!(body.is_empty());
//     }

//     #[actix_web::test]
//     async fn test_list_artifacts_with_meta() {
//         let _ = fs::remove_dir_all("./data");
//         fs::create_dir_all("./data").unwrap();

//         let meta = ArtifactMetadata {
//             id: "test-id".into(),
//             original_filename: "file.txt".into(),
//             size: 123,
//             sha256: "abc".into(),
//             upload_time: "2025-01-01T00:00:00Z".into(),
//             metadata: "meta".into(),
//         };
//         fs::write("./data/test-id.json", serde_json::to_string(&meta).unwrap()).unwrap();

//         let app = test::init_service(
//             App::new().service(list_artifacts)
//         ).await;
//         let req = test::TestRequest::get().uri("/artifacts").to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(resp.status(), StatusCode::OK);

//         let body: Vec<ArtifactMetadata> = test::read_body_json(resp).await;
//         assert_eq!(body.len(), 1);
//         assert_eq!(body[0], meta);
//     }

//     #[actix_web::test]
//     async fn test_upload_and_download_artifact() {
//         // Reset data directory
//         let _ = fs::remove_dir_all("./data");
//         fs::create_dir_all("./data").unwrap();

//         // Prepare multipart payload
//         let boundary = "TESTBOUNDARY";
//         let payload = format!(
//             "--{boundary}\r\n\
//              Content-Disposition: form-data; name=\"file\"; filename=\"hello.txt\"\r\n\
//              \r\n\
//              Hello, world!\r\n\
//              --{boundary}--\r\n",
//             boundary=boundary
//         );
//         let content_type = format!("multipart/form-data; boundary={}", boundary);

//         let app = test::init_service(
//             App::new()
//                 .service(upload_artifact)
//                 .service(download_artifact)
//         ).await;

//         // Upload
//         let req = test::TestRequest::post()
//             .uri("/upload")
//             .insert_header(("Content-Type", content_type.clone()))
//             .set_payload(payload.clone())
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(resp.status(), StatusCode::OK);

//         let body_bytes = test::read_body(resp).await;
//         let id = str::from_utf8(&body_bytes).unwrap();
//         assert!(!id.is_empty());

//         // Verify metadata file
//         let meta_path = format!("./data/{}.json", id);
//         let meta_contents = fs::read_to_string(&meta_path).unwrap();
//         let meta: ArtifactMetadata = serde_json::from_str(&meta_contents).unwrap();
//         assert_eq!(meta.id, id);
//         assert_eq!(meta.original_filename, "hello.txt");
//         assert_eq!(meta.size, "Hello, world!".len() as u64);

//         // Download
//         let req = test::TestRequest::get()
//             .uri(&format!("/download/{}", id))
//             .to_request();
//         let mut resp = test::call_service(&app, req).await;
//         assert_eq!(resp.status(), StatusCode::OK);

//         let mut downloaded = Vec::new();
//         while let Some(chunk) = resp.take_body().next().await {
//             downloaded.extend_from_slice(&chunk.unwrap());
//         }
//         assert_eq!(downloaded, b"Hello, world!");
//     }
// }