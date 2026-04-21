// Compliance Models
// Data structures for tracking legal and regulatory compliance

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "policy_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyType {
    #[serde(rename = "PRIVACY_POLICY")]
    PrivacyPolicy,
    #[serde(rename = "TERMS_OF_SERVICE")]
    TermsOfService,
    #[serde(rename = "DATA_PROCESSING_AGREEMENT")]
    DataProcessingAgreement,
}

impl PolicyType {
    pub fn as_str(&self) -> &str {
        match self {
            PolicyType::PrivacyPolicy => "PRIVACY_POLICY",
            PolicyType::TermsOfService => "TERMS_OF_SERVICE",
            PolicyType::DataProcessingAgreement => "DATA_PROCESSING_AGREEMENT",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PolicyAgreement {
    pub id: i64,
    pub merchant_id: i64,
    pub policy_type: String, // Stored as string in the DB for flexibility
    pub version: String,
    pub accepted_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyAgreement {
    pub merchant_id: i64,
    pub policy_type: PolicyType,
    pub version: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
