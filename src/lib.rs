use clap::Parser;
use std::{collections::HashMap, net::Ipv4Addr, sync::Arc, time::Instant};
use tokio::sync::RwLock;

mod crd;

pub type BackupCrd = crd::Backup;
pub type SharedAppState = Arc<AppState>;

/// MariaDB Operator Backup Metrics - Exposes Prometheus metrics from Backup CRD's
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The port to listen on.
    #[arg(
        long,
        default_value = "0.0.0.0",
        env = "MARIADB_OPERATOR_BACKUP_METRICS_BIND_ADDR"
    )]
    pub bind_address: Ipv4Addr,

    /// The port to listen on.
    #[arg(
        long,
        default_value_t = 80,
        value_parser = clap::value_parser!(u16).range(1..),
        env = "MARIADB_OPERATOR_BACKUP_METRICS_BIND_PORT"
    )]
    pub bind_port: u16,
}

#[derive(Debug)]
pub struct AppState {
    health: RwLock<AppHealth>,
    args: Args,
    cache: RwLock<HashMap<String, BackupCrd>>,
}

impl AppState {
    /// Create a new AppState
    pub fn new(args: Args) -> Self {
        Self {
            health: RwLock::new(AppHealth::default()),
            args,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get the cache
    pub async fn cache(&self) -> HashMap<String, BackupCrd> {
        self.cache.read().await.clone()
    }

    /// Update the cache
    pub async fn update_cache(&self, cache: HashMap<String, BackupCrd>) {
        *self.cache.write().await = cache;
    }

    /// Get the health status
    pub async fn health(&self) -> AppHealth {
        self.health.read().await.clone()
    }

    /// Update the health status
    pub async fn update_health(&self, health: AppHealth) {
        *self.health.write().await = health;
    }

    /// Get the arguments
    pub fn args(&self) -> &Args {
        &self.args
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppHealth {
    pub last_run: Option<Instant>,
}

impl AppHealth {
    /// Update the last run time
    pub fn update(&mut self) {
        self.last_run = Some(Instant::now());
    }

    /// Check if the application is healthy
    pub fn is_healthy(&self) -> bool {
        if let Some(last_run) = self.last_run {
            last_run.elapsed().as_secs() < 60
        } else {
            false
        }
    }
}
