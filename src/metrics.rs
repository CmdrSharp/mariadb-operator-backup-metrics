use crate::AppState;
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

            let status = crd.status.unwrap();

            // Find an existing, or create a new metric
            let mut metric = self
                .find_or_create_metric(("backup_last_run_status".into(), name.clone()))
                .await;

            // Insert the reason (if it exists on the status)
            metric
                .labels
                .insert("reason".into(), status.reason().unwrap_or("Unknown".into()));

            // Set the value of the metric based on the status
            if status.success() {
                metric.value = 1;
            }

            if status.failed() {
                metric.value = 0;
            }

            // Insert the metric back into the cache
            self.metrics
                .insert(("backup_last_run_status".into(), name.clone()), metric);
        }

        self.render().await;
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
