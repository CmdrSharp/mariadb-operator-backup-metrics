use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[kube(
    group = "k8s.mariadb.com",
    version = "v1alpha1",
    kind = "Backup",
    status = "BackupStatus"
)]
#[serde(rename_all = "camelCase")]
pub struct BackupSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub databases: Option<Vec<String>>,
    pub maria_db_ref: MariaDbRef,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<BackupCondition>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct BackupCondition {
    pub last_transition_time: Option<String>,
    pub message: Option<String>,
    pub reason: Option<String>,
    pub status: String,
    pub type_: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct MariaDbRef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}
