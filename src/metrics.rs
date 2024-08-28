use crate::{crd::BackupStatus, AppState};
use std::collections::HashMap;

type MetricName = String;
type MetricValue = usize;
type MetricNameLabel = String;
type MetricKey = (MetricName, MetricNameLabel);
type Labels = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct GaugeMetric {
    labels: Labels,
    value: MetricValue,
}

#[derive(Debug, Clone, Default)]
pub struct MetricCache {
    metrics: HashMap<MetricKey, GaugeMetric>,
    rendered: String,
}

impl MetricCache {
    /// Generate metrics
    pub async fn generate(&mut self, state: &AppState) {
        let cache = state.cache().await;

        for (name, crd) in cache {
            if crd.status.is_none() {
                continue;
            }

            let crd_status = crd.status.unwrap();

            // Generate metrics
            let status = self.status(name.clone(), &crd_status).await;
            let timestamp = self.timestamp(name.clone(), &crd_status).await;

            // Insert the metric back into the cache
            self.metrics
                .insert(("backup_last_run_status".into(), name.clone()), status);

            if let Some(timestamp) = timestamp {
                self.metrics.insert(
                    ("backup_last_run_timestamp".into(), name.clone()),
                    timestamp,
                );
            }
        }

        self.render().await;
    }

    /// Generate a status metric
    async fn status(&self, name: String, status: &BackupStatus) -> GaugeMetric {
        let mut metric = self
            .find_or_create_metric(("backup_last_run_status".into(), name.clone()))
            .await;

        metric
            .labels
            .insert("reason".into(), status.reason().unwrap_or("Unknown".into()));

        if status.success() {
            metric.value = 1;
        }

        if status.failed() {
            metric.value = 0;
        }

        metric
    }

    /// Generate a timestamp metric
    async fn timestamp(
        &self,
        name: String,
        status: &BackupStatus,
    ) -> Option<GaugeMetric> {
        let mut metric = self
            .find_or_create_metric(("backup_last_run_timestamp".into(), name.clone()))
            .await;

        if status.success() {
            if let Some(timestamp) = status.last_run() {
                metric.value = timestamp as usize;
            }
        }

        None
    }

    /// Get the rendered metrics
    pub fn rendered(&self) -> String {
        self.rendered.clone()
    }

    /// Render metrics
    async fn render(&mut self) {
        let mut rendered = String::new();

        for ((metric_name, resource_name), metric) in &self.metrics {
            rendered.push_str(&format!("# TYPE {} gauge\n", metric_name));

            rendered.push_str(&format!(
                "{}{{{} {}}} {}\n",
                metric_name,
                metric
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<String>>()
                    .join(","),
                resource_name,
                metric.value
            ));
        }

        self.rendered = rendered;
    }

    /// Find a metric by key
    async fn find_or_create_metric(&self, key: MetricKey) -> GaugeMetric {
        self.metrics.get(&key).cloned().unwrap_or(GaugeMetric {
            labels: vec![("name".into(), key.1.clone())].into_iter().collect(),
            value: 0,
        })
    }
}
