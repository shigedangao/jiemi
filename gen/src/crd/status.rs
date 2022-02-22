use std::collections::VecDeque;
use std::string::ToString;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use kube::{
    Client,
    Api,
    api::PostParams
};
use super::Decryptor;
use crate::err::Error;

// constant
const MAX_QUEUE_SIZE: usize = 10;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct DecryptorStatus {
    pub current: Status,
    history: Option<VecDeque<Status>>,
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Sync,
    Unsync,
    Error
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
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
    /// * `revision` - String
    /// * `decryptor` - Decryptor
    pub fn new(
        status: SyncStatus,
        err: Option<String>,
        revision: String,
        decryptor: Decryptor,
    ) -> Self {
        let (history, previous_id) = match decryptor.status {
            Some(mut prev) => {
                prev.add_current_to_history();
                (prev.history, prev.current.id)
            },
            None => (Some(VecDeque::new()), 0 as u64)
        };

        let file_to_decrypt = decryptor.spec.source.file_to_decrypt;
        DecryptorStatus {
            current: Status::new(status, revision, file_to_decrypt, err, previous_id),
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

    /// Update the status of an existing Decryptor crd
    /// 
    /// # Arguments
    /// * `self` - Self
    /// * `name` - &str
    /// * `ns` - &str
    pub async fn update_status(self, name: &str, ns: &str) -> Result<(), Error> {
        let client = Client::try_default().await?;
        let api = Api::<Decryptor>::namespaced(client.clone(), ns);
        let mut curr_decryptor_status = api.get_status(&name).await?;
        curr_decryptor_status.status = Some(self);

        api.replace_status(
            &name,
            &PostParams::default(),
            serde_json::to_vec(&curr_decryptor_status)?
        ).await?;

        Ok(())
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
            deployed_at: Utc::now().to_rfc3339().to_string(),
            id: previous_id + 1,
            revision,
            file_to_decrypt,
            status,
            error_message: err.and_then(|msg| Some(msg.to_string()))
        }
    }
}