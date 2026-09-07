use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

mod handlers;
mod mailer;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:comments.db?mode=rwc".to_string());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let admin_routes = Router::new()
        .route("/admin", get(handlers::admin_dashboard))
        .route("/admin/api/comments", get(handlers::list_admin_comments))
        .route("/admin/api/comments/:id/toggle", post(handlers::toggle_approve_comment))
        .route("/admin/api/comments/:id", delete(handlers::delete_comment))
        .route_layer(middleware::from_fn(handlers::require_admin_auth));

    // 🌟 NEW: Unprotected routes for login/logout
    let public_admin_routes = Router::new()
        .route("/admin/login", get(handlers::admin_login_form))
        .route("/admin/login", post(handlers::admin_login_submit))
        .route("/admin/logout", get(handlers::admin_logout));

    let app = Router::new()
        .route("/widget.js", get(handlers::serve_js))
        .route("/widget.css", get(handlers::serve_css))
        .route("/api/comments", get(handlers::get_comments))
        .route("/api/comments", post(handlers::post_comment))
        .merge(public_admin_routes)
        .merge(admin_routes)
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Comment service running on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}