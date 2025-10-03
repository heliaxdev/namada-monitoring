use std::time::Duration;

use serde::Deserialize;

#[derive(Default, Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd)]
pub enum Severity {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn to_emoji(&self) -> String {
        match self {
            Severity::Low => ":large_yellow_circle:".to_string(),
            Severity::Medium => ":large_orange_circle:".to_string(),
            Severity::High => ":large_red_square:".to_string(),
            Severity::Critical => ":skull:".to_string(),
        }
    }

    pub fn to_color(&self) -> String {
        match self {
            Severity::Low => "#ffffff".to_string(),
            Severity::Medium => "#ffff00".to_string(),
            Severity::High => "#ffa500".to_string(),
            Severity::Critical => "#ff0000".to_string(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Alert {
    pub check_id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub metadata: Metadata,
    pub trigger_after: Option<Duration>,
    pub continous: bool,
}

#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub block_height: Option<u32>,
    pub tx_id: Option<String>,
}

impl Metadata {
    pub fn new(block_height: Option<u32>, tx_id: Option<String>) -> Self {
        Self {
            block_height,
            tx_id,
        }
    }
}
