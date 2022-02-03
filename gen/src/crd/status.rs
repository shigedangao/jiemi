use std::collections::VecDeque;
use std::string::ToString;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

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
    /// * `filename` - String
    /// * `previous` - Option<DecryptorStatus>
    /// * `revision` - String
    pub fn new(
        status: SyncStatus,
        err: Option<String> ,
        filename: String,
        previous: Option<DecryptorStatus>,
        revision: String
    ) -> Self {
        let (history, previous_id) = match previous {
            Some(mut prev) => {
                prev.add_current_to_history();
                (prev.history, prev.current.id)
            },
            None => (Some(VecDeque::new()), 0 as u64)
        };

        DecryptorStatus {
            current: Status::new(status, revision, filename, err, previous_id),
            history
        }
    }

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
    fn new(
        status: SyncStatus,
        revision: String,
        filename: String,
        err: Option<String>,
        previous_id: u64
    ) -> Self {
        Status {
            deployed_at: "2022-10-10:15h30:05".to_owned(),
            id: previous_id + 1,
            revision,
            filename,
            status,
            error_message: err.and_then(|msg| Some(msg.to_string()))
        }
    }
}