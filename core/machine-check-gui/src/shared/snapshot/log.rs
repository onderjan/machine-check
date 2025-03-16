use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StepStatus {
    Completed,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepMessage {
    pub status: StepStatus,
    pub num_refinements: u64,
    pub duration: Duration,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    Error(String),
    Step(StepMessage),
    Reset,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub ty: MessageType,
    pub time: SystemTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Log {
    pub messages: Vec<Message>,
}

impl Log {
    pub fn new() -> Log {
        Log {
            messages: Vec::new(),
        }
    }

    pub fn error(&mut self, msg: String) {
        self.add_message(MessageType::Error(msg));
    }

    pub fn add_message(&mut self, msg: MessageType) {
        let time = SystemTime::now();
        self.messages.push(Message { ty: msg, time });
    }
}

impl Default for Log {
    fn default() -> Self {
        Self::new()
    }
}
