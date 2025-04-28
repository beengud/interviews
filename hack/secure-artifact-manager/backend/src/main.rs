use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use sqlx::postgres::PgPoolOptions;
use aws_sdk_s3::Client;
use aws_config::meta::region::RegionProviderChain;
use backend::{
    configure_routes,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::fs::create_dir_all("./data")?;
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Starting backend on http://0.0.0.0:8080");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    let region_provider = RegionProviderChain::default_provider()
        .or_else("us-east-1");

    let shared_config = aws_config::from_env()
        .region(region_provider)
        .endpoint_url("http://minio:9000")
        .load()
        .await;

    let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
        .force_path_style(true)
        .build();

    let s3_client = Client::from_conf(s3_config);

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(db_pool.clone()))
        .app_data(web::Data::new(s3_client.clone()))
        .configure(configure_routes)
        .wrap(Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600),
        )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
