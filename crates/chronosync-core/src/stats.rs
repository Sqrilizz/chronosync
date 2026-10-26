use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectSummary {
    pub name: String,
    pub total_seconds: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguageSummary {
    pub name: String,
    pub total_seconds: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailySummary {
    pub date: String,
    pub total_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatsSummary {
    pub total_seconds: u64,
    pub projects: Vec<ProjectSummary>,
    pub languages: Vec<LanguageSummary>,
    pub daily_activity: Vec<DailySummary>,
}
