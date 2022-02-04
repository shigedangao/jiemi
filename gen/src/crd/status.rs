use std::collections::VecDeque;
use std::string::ToString;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use super::Decryptor;

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
    revision: String,
    filename: String,
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

        let filename = decryptor.spec.source.filename;
        DecryptorStatus {
            current: Status::new(status, revision, filename, err, previous_id),
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
    /// * `filename` - String
    /// * `err` - Option<String>
    /// * `previous_id` - u64
    fn new(
        status: SyncStatus,
        revision: String,
        filename: String,
        err: Option<String>,
        previous_id: u64
    ) -> Self {
        Status {
            deployed_at: Utc::now().to_rfc3339().to_string(),
            id: previous_id + 1,
            revision,
            filename,
            status,
            error_message: err.and_then(|msg| Some(msg.to_string()))
        }
    }
}