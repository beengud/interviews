use actix_web::{test, App, http::StatusCode};
use futures_util::stream::StreamExt as _;
use std::{fs, str};
use backend::{
    list_artifacts,
    upload_artifact,
    download_artifact,
    ArtifactMetadata,
};

#[actix_web::test]
async fn list_artifacts_empty() {
    let _ = fs::remove_dir_all("./data");
    fs::create_dir_all("./data").unwrap();

    let app = test::init_service(App::new().service(list_artifacts)).await;
    let req = test::TestRequest::get().uri("/artifacts").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<ArtifactMetadata> = test::read_body_json(resp).await;
    assert!(body.is_empty());
}

#[actix_web::test]
async fn list_artifacts_with_meta() {
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

    let app = test::init_service(App::new().service(list_artifacts)).await;
    let req = test::TestRequest::get().uri("/artifacts").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<ArtifactMetadata> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0], meta);
}

#[actix_web::test]
async fn upload_and_download_artifact() {
    let _ = fs::remove_dir_all("./data");
    fs::create_dir_all("./data").unwrap();

    // build multipart payload
    let boundary = "TESTBOUNDARY";
    let payload = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"hello.txt\"\r\n\
         \r\n\
         Hello, world!\r\n\
         --{boundary}--\r\n",
        boundary = boundary
    );
    let content_type = format!("multipart/form-data; boundary={}", boundary);

    let app = test::init_service(
        App::new()
            .service(upload_artifact)
            .service(download_artifact)
    ).await;

    // upload
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

    // verify metadata
    let meta_contents = fs::read_to_string(&format!("./data/{}.json", id)).unwrap();
    let meta: ArtifactMetadata = serde_json::from_str(&meta_contents).unwrap();
    assert_eq!(meta.id, id);
    assert_eq!(meta.original_filename, "hello.txt");
    assert_eq!(meta.size, "Hello, world!".len() as u64);

    // download
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