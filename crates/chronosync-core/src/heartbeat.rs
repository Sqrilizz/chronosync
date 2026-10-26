use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heartbeat {
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    pub timestamp: i64,
    #[serde(default)]
    pub is_write: bool,
}

impl Heartbeat {
    pub fn new(project: impl Into<String>, timestamp: i64) -> Self {
        Self {
            project: project.into(),
            branch: None,
            language: None,
            file: None,
            editor: None,
            timestamp,
            is_write: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HeartbeatPayload {
    Batch(Vec<Heartbeat>),
    Single(Heartbeat),
}

impl HeartbeatPayload {
    pub fn into_vec(self) -> Vec<Heartbeat> {
        match self {
            Self::Single(hb) => vec![hb],
            Self::Batch(list) => list,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestResponse {
    pub status: String,
    pub accepted: usize,
    pub total: usize,
}
