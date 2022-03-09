use std::collections::VecDeque;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use super::Decryptor;

// constant
const MAX_QUEUE_SIZE: usize = 10;

/// Status field of the CRD. It represent the Sync status of the CRD. See below to see how it looks
/// 
/// # Example
/// Status:
///     Current:
///         deployed_at:      2022-03-03T20:37:59.024362965+00:00
///         error_message:    <nil>
///         file_to_decrypt:  pgp/secret.enc.yaml
///         Id:               1
///         Revision:         a888f02e1111beb2c543d729faa5d516ecaa9e12
///         Status:  Sync
///     History:
///         List of previous statuses...
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct DecryptorStatus {
    pub current: Status,
    history: Option<VecDeque<Status>>,
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Sync,
    Unsync,
    Error
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize, PartialEq)]
pub struct Status {
    deployed_at: String,
    id: u64,
    pub revision: String,
    file_to_decrypt: String,
    status: SyncStatus,
    error_message: Option<String>
}

impl DecryptorStatus {
    /// Create a new Decryptor Status struct. This status is used by the Controller to update the k8s status
    /// 
    /// # Arguments
    /// * `status` - SyncStatus
    /// * `err` - Option<String>
    /// * `revision` - Option<String>
    /// * `current` - Decryptor
    pub fn new(
        status: SyncStatus,
        err: Option<String>,
        revision: Option<String>,
        current: &Decryptor,
    ) -> Self {
        let (history, previous_id) = match current.status.to_owned() {
            Some(mut prev) => {
                prev.add_current_to_history();
                (prev.history, prev.current.id)
            },
            None => (Some(VecDeque::new()), 0_u64)
        };

        let file_to_decrypt = current.spec.source.file_to_decrypt.to_owned();
        DecryptorStatus {
            current: Status::new(
                status,
                revision.unwrap_or_default(),
                file_to_decrypt, 
                err,
                previous_id
            ),
            history
        }
    }

    /// Update the history of the status by adding the current struct status
    /// to the history. The current status will then be replaced with a new one...
    /// 
    /// # Arguments
    /// * `&mut self` - Self
    fn add_current_to_history(&mut self) {
        if let Some(queue) = self.history.as_mut() {
            if queue.len() > MAX_QUEUE_SIZE {
                queue.pop_front();
            }

            queue.push_back(self.current.clone());
        }
    }
}

impl Status {
    /// Create a new Status
    /// 
    /// # Arguments
    /// * `status` - SyncStatus
    /// * `revision` - String
    /// * `file_to_decrypt` - String
    /// * `err` - Option<String>
    /// * `previous_id` - u64
    fn new(
        status: SyncStatus,
        revision: String,
        file_to_decrypt: String,
        err: Option<String>,
        previous_id: u64
    ) -> Self {
        Status {
            deployed_at: Utc::now().to_rfc3339(),
            id: previous_id + 1,
            revision,
            file_to_decrypt,
            status,
            error_message: err
        }
    }
}

#[cfg(test)]
mod tests {
    use kube::core::ObjectMeta;
    use kube::{Client, Api};
    use crate::crd::{DecryptorSpec, Provider, Source};
    use crate::crd::repo::Repository;
    use super::*;

    fn get_decryptor() -> Decryptor {
        Decryptor {
            metadata: ObjectMeta::default(),
            spec: DecryptorSpec {
                provider: Provider {
                    gcp: None,
                    aws: None,
                    pgp: None
                },
                source: Source {
                    repository: Repository {
                        url: "https://foo.bar".to_owned(),
                        credentials: None
                    },
                    file_to_decrypt: "foo".to_owned(),
                    sops_path: "bar".to_owned()
                }
            },
            status: None
        }
    }

    #[test]
    fn expect_to_create_status_wo_history() {
        let decryptor = get_decryptor();
        let status = DecryptorStatus::new(
            SyncStatus::Sync, 
            None, 
            Some("foo".to_owned()),
            &decryptor
        );
        
        assert_eq!(status.current.id, 1);
        assert_eq!(status.current.revision, "foo");
        assert_eq!(status.current.file_to_decrypt, "foo");
        assert_eq!(status.current.error_message, None);
        assert_eq!(status.history, Some(VecDeque::new()));
    }

    #[test]
    fn expect_to_create_status_with_history() {
        let mut decryptor = get_decryptor();
        let status = DecryptorStatus::new(
            SyncStatus::Sync, 
            None, 
            Some("foo".to_owned()),
            &decryptor
        );

        decryptor.status = Some(status);

        let new_status = DecryptorStatus::new(
            SyncStatus::Sync, 
            None, 
            Some("bar".to_owned()),
            &decryptor
        );

        assert!(new_status.history.is_some());
        let history = new_status.history.unwrap();
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn expect_to_update_decryptor_status_on_cluster() {
        let client = Client::try_default().await.unwrap();
        let api: Api<Decryptor> = Api::namespaced(client, "default");

        let mut decryptor = api.get("miwen-pgp-test-decryptor").await.unwrap();
        let status = DecryptorStatus::new(
            SyncStatus::Unsync, 
            None, 
            Some("foo".to_owned()),
            &decryptor
        );
        
        decryptor.set_status(status);
        let status = decryptor.update_status().await;
        assert!(status.is_ok());
    }
}