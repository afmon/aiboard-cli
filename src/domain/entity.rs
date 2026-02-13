use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreadStatus {
    #[default]
    Open,
    Closed,
}

impl std::fmt::Display for ThreadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadStatus::Open => write!(f, "open"),
            ThreadStatus::Closed => write!(f, "closed"),
        }
    }
}

impl std::str::FromStr for ThreadStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(ThreadStatus::Open),
            "closed" => Ok(ThreadStatus::Closed),
            other => Err(format!("unknown thread status: {}", other)),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreadPhase {
    Planning,
    Implementing,
    Reviewing,
    Done,
}

impl std::fmt::Display for ThreadPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadPhase::Planning => write!(f, "planning"),
            ThreadPhase::Implementing => write!(f, "implementing"),
            ThreadPhase::Reviewing => write!(f, "reviewing"),
            ThreadPhase::Done => write!(f, "done"),
        }
    }
}

impl std::str::FromStr for ThreadPhase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "planning" => Ok(ThreadPhase::Planning),
            "implementing" => Ok(ThreadPhase::Implementing),
            "reviewing" => Ok(ThreadPhase::Reviewing),
            "done" => Ok(ThreadPhase::Done),
            other => Err(format!("unknown thread phase: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,
    pub name: Option<String>,
    pub title: String,
    pub source_url: Option<String>,
    pub status: ThreadStatus,
    pub phase: Option<ThreadPhase>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub thread_id: String,
    pub session_id: Option<String>,
    pub sender: Option<String>,
    pub role: Role,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub parent_id: Option<String>,
    pub source: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::System => write!(f, "system"),
            Role::Tool => write!(f, "tool"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            "system" => Ok(Role::System),
            "tool" => Ok(Role::Tool),
            other => Err(format!("unknown role: {}", other)),
        }
    }
}
