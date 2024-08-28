use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use color_eyre::eyre::Result;
use mariadb_operator_backup_metrics::SharedAppState;
use std::net::SocketAddr;

/// Start the web server
pub async fn start_server(state: SharedAppState) -> Result<()> {
    tracing::info!("Starting web server");

    let addr: SocketAddr = format!("{}:{}", state.args().bind_address, state.args().bind_port)
        .parse()
        .expect("Unable to parse address");

    let app = Router::new()
        .route("/alive", get(alive))
        .route("/health", get(health))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

/// Returns OK if the web server is alive, used for Kubernetes Aliveness Check
async fn alive() -> impl IntoResponse {
    StatusCode::OK
}

/// Returns OK if the application is healthy and has updated its internal state recently
async fn health(State(state): State<SharedAppState>) -> impl IntoResponse {
    let health = state.health().await;

    if health.is_healthy() {
        return (StatusCode::OK, format!("Last run: {:?}", health.last_run));
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        format!("Last run: {:?}", health.last_run),
    )
}
