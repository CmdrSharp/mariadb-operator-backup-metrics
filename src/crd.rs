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

impl BackupStatus {
    /// Checks if the last transition state for the backup is successful
    pub fn success(&self) -> bool {
        if self.conditions.is_none() {
            return false;
        }

        self.conditions
            .as_ref()
            .unwrap()
            .iter()
            .filter(|c| c.type_ == "Complete" && c.status == "True")
            .count()
            > 0
    }

    /// Checks if the transition state indicates a scheduled backup
    pub fn scheduled(&self) -> bool {
        if self.conditions.is_none() {
            return false;
        }

        self.conditions
            .as_ref()
            .unwrap()
            .iter()
            .filter(|c| {
                c.type_ == "Complete"
                    && c.status == "False"
                    && c.message == Some("Scheduled".into())
            })
            .count()
            > 0
    }

    /// Checks if a backup is running
    pub fn running(&self) -> bool {
        if self.conditions.is_none() {
            return false;
        }

        self.conditions
            .as_ref()
            .unwrap()
            .iter()
            .filter(|c| {
                c.type_ == "Complete"
                    && c.status == "False"
                    && c.message == Some("Running".into())
            })
            .count()
            > 0
    }

    /// Checks if a backup has likely failed
    pub fn failed(&self) -> bool {
        !self.running() && !self.success() && !self.scheduled()
    }

    /// Get the reason for the last transition state
    pub fn reason(&self) -> Option<String> {
        self.conditions
            .as_ref()?
            .iter()
            .find(|c| c.type_ == "Complete")
            .and_then(|c| c.reason.clone())
    }
}
