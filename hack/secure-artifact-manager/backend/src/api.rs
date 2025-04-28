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

    pub scan_status: String,
}

#[get("/artifacts")]
pub async fn list_artifacts(db_pool: web::Data<PgPool>) -> impl Responder {
    // Query artifact metadata from PostgreSQL database
    let artifacts = sqlx::query_as::<_, ArtifactMetadata>(
        r#"
        SELECT id, original_filename, size_bytes, sha256, upload_time, metadata, scan_status
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
        scan_status: "in-progress".to_string(),
    };

    // Insert metadata into PostgreSQL database
    let insert_result = sqlx::query(
        r#"
        INSERT INTO artifacts (id, original_filename, size_bytes, sha256, upload_time, metadata, scan_status)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#)
        .bind(&meta.id)
        .bind(&meta.original_filename)
        .bind(meta.size_bytes as i64)
        .bind(&meta.sha256)
        .bind(&meta.upload_time)
        .bind(&meta.metadata)
        .bind(&meta.scan_status)
    .execute(db_pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        log::error!("Failed to insert artifact metadata into DB: {}", e);
        return HttpResponse::InternalServerError().body("Failed to save artifact metadata");
    }

    // Trigger a mock scan immediately after upload
    let scan_result = sqlx::query(
        r#"
        UPDATE artifacts
        SET scan_status = $1
        WHERE id = $2
        "#)
        .bind("clean")
        .bind(&id)
        .execute(db_pool.get_ref())
        .await;

    if let Err(e) = scan_result {
        log::error!("Failed to update scan status after upload: {}", e);
    } else {
        log::info!("Artifact {} automatically scanned and marked as clean", id);
    }

    log::info!("Uploaded file: {} (saved as: {})", original_filename, id);
    HttpResponse::Ok().body(id)
}

#[post("/scan/{id}")]
pub async fn scan_artifact(
    path: web::Path<String>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let id = path.into_inner();

    // Mock scan: update scan_status to "clean"
    let update_result = sqlx::query(
        r#"
        UPDATE artifacts
        SET scan_status = $1
        WHERE id = $2
        "#)
        .bind("clean")
        .bind(&id)
        .execute(db_pool.get_ref())
        .await;

    match update_result {
        Ok(_) => {
            log::info!("Artifact {} marked as clean after scan", id);
            HttpResponse::Ok().body(format!("Artifact {} scanned and marked as clean", id))
        },
        Err(e) => {
            log::error!("Failed to update scan status for artifact {}: {}", id, e);
            HttpResponse::InternalServerError().body("Failed to update scan status")
        }
    }
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
        .service(download_artifact)
        .service(scan_artifact);
}
