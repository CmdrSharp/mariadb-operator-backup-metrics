use color_eyre::Result;
use either::Either;
use futures::TryStreamExt;
use kube::runtime::{watcher, WatchStreamExt};
use kube::{api::ListParams, Api, Client, ResourceExt};
use mariadb_operator_backup_metrics::{BackupCrd, SharedAppState};
use std::time::Duration;
use tokio::time;

/// Start watcher for backup resources
pub async fn watch(state: SharedAppState) -> Result<()> {
    tracing::info!("Starting watcher for backup resources");

    let client = Client::try_default().await?;
    let api: Api<BackupCrd> = Api::all(client);

    // Initial scan for existing resources
    if let Ok(list) = api.list(&ListParams::default()).await {
        for item in list {
            tracing::debug!("Found resource: {}", item.name_any());

            // Add the item to the cache
            let mut cache = state.cache().await;
            cache.insert(item.name_any(), item);
            state.update_cache(cache).await;

            // Trigger a re-generation of metrics
            let mut metrics = state.metrics().await;
            metrics.generate(&state).await;
            state.update_metrics(metrics).await;
        }
    }

    tokio::spawn(async move {
        loop {
            let api: Api<BackupCrd> = api.clone();

            match _watch(api, state.clone()).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Error watching for backup resources: {:?}", e);
                }
            }
        }
    });

    Ok(())
}

/// Watch for changes to backup resources
async fn _watch(api: Api<BackupCrd>, state: SharedAppState) -> Result<()> {
    tracing::info!("Watching for backup resources");

    let mut watcher_stream =
        Box::pin(watcher(api, watcher::Config::default()).applied_objects());
    let mut interval = time::interval(Duration::from_secs(10));

    loop {
        let event = tokio::select! {
            event = watcher_stream.try_next() => {
                match event {
                    Ok(Some(resource)) => Either::Left(resource),
                    Ok(None) => break,
                    Err(e) => return Err(e.into()),
                }
            },
            _ = interval.tick() => {
                Either::Right(())
            },
        };

        match event {
            Either::Left(resource) => {
                tracing::debug!("Event: {:?}", resource);

                // Update the item in the cache
                let mut cache = state.cache().await;
                cache.insert(resource.name_any(), resource);
                state.update_cache(cache).await;
            }
            Either::Right(_) => {
                tracing::debug!("Interval tick");

                // Trigger a re-generation of metrics
                let mut metrics = state.metrics().await;
                metrics.generate(&state).await;
                state.update_metrics(metrics).await;

                // Update the AppHealth
                let mut health = state.health().await;
                health.update();
                state.update_health(health).await;
            }
        }
    }

    Ok(())
}
