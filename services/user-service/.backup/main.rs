mod handlers;
mod models;
mod openapi;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Export OpenAPI spec to YAML file (only with --features export-spec)
#[cfg(feature = "export-spec")]
fn export_spec() -> std::io::Result<()> {
    use std::path::Path;
    
    // Serialize OpenAPI to YAML using serde_yaml
    let openapi = openapi::ApiDoc::openapi();
    let yaml = serde_yaml::to_string(&openapi)
        .expect("Failed to serialize OpenAPI to YAML");
    
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shared/openapi/user.yaml"
    ));
    
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, yaml)?;
    
    println!("cargo:warning=OpenAPI spec exported to {:?}", path);
    Ok(())
}

#[tokio::main]
async fn main() {
    // Export OpenAPI spec if feature is enabled
    #[cfg(feature = "export-spec")]
    {
        export_spec().expect("Failed to export OpenAPI spec");
        tracing::info!("OpenAPI spec exported to shared/openapi/user.yaml");
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build API routes
    let api_routes = Router::new()
        .route("/api/v1/auth/register", post(handlers::register))
        .route("/api/v1/auth/login", post(handlers::login))
        .route("/api/v1/auth/refresh", post(handlers::refresh_token))
        .route("/api/v1/users", get(handlers::list_users));

    // Build application with routes and Swagger UI
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🚀 User Service listening on http://{}", addr);
    tracing::info!("📚 Swagger UI available at http://{}/docs", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
