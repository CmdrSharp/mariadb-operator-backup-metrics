use clap::Parser;
use color_eyre::eyre::Result;
use mariadb_operator_backup_metrics::AppState;
use std::sync::Arc;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod crd;
mod server;
mod watcher;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "mariadb_operator_backup_metrics=info,tower_http=info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = mariadb_operator_backup_metrics::Args::parse();
    let state = Arc::new(AppState::new(args));

    // Thread for handling signals
    tokio::spawn(async move {
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to register SIGTERM");
        let mut sigint =
            signal(SignalKind::interrupt()).expect("Failed to register SIGINT");

        select! {
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM, shutting down");
                std::process::exit(0);
            }
            _ = sigint.recv() => {
                tracing::info!("Received SIGINT, shutting down");
                std::process::exit(0);
            }
        }
    });

    // Start the watcher for CRD's
    match watcher::watch(state.clone()).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error watching for backup resources: {:?}", e);
        }
    }

    // Start the web server
    server::start_server(state.clone()).await?;

    Ok(())
}
