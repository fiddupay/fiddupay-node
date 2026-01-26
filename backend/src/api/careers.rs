use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct JobPosting {
    pub id: String,
    pub title: String,
    pub department: String,
    pub location: String,
    pub employment_type: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub benefits: Vec<String>,
    pub posted_date: String,
}

#[derive(Serialize, Deserialize)]
pub struct CareersResponse {
    pub jobs: Vec<JobPosting>,
    pub company_info: CompanyInfo,
}

#[derive(Serialize, Deserialize)]
pub struct CompanyInfo {
    pub name: String,
    pub description: String,
    pub values: Vec<String>,
    pub benefits: Vec<String>,
}

pub async fn get_careers(
    State(_state): State<AppState>,
) -> Result<Json<CareersResponse>, StatusCode> {
    let jobs = vec![
        JobPosting {
            id: "1".to_string(),
            title: "Senior Blockchain Developer".to_string(),
            department: "Engineering".to_string(),
            location: "Remote".to_string(),
            employment_type: "Full-time".to_string(),
            description: "Build and maintain our multi-chain payment infrastructure".to_string(),
            requirements: vec![
                "5+ years blockchain development experience".to_string(),
                "Proficiency in Rust, Solidity, or similar".to_string(),
                "Experience with DeFi protocols".to_string(),
            ],
            benefits: vec![
                "Competitive salary + equity".to_string(),
                "Remote work flexibility".to_string(),
                "Health insurance".to_string(),
            ],
            posted_date: "2026-01-20T00:00:00Z".to_string(),
        },
        JobPosting {
            id: "2".to_string(),
            title: "Frontend Engineer".to_string(),
            department: "Engineering".to_string(),
            location: "Remote".to_string(),
            employment_type: "Full-time".to_string(),
            description: "Create beautiful, user-friendly payment interfaces".to_string(),
            requirements: vec![
                "3+ years React/TypeScript experience".to_string(),
                "Experience with modern frontend tools".to_string(),
                "Understanding of web3 concepts".to_string(),
            ],
            benefits: vec![
                "Competitive salary + equity".to_string(),
                "Remote work flexibility".to_string(),
                "Professional development budget".to_string(),
            ],
            posted_date: "2026-01-18T00:00:00Z".to_string(),
        },
    ];

    let company_info = CompanyInfo {
        name: "FidduPay".to_string(),
        description: "Leading cryptocurrency payment gateway trusted by businesses worldwide".to_string(),
        values: vec![
            "Trust and Security".to_string(),
            "Innovation".to_string(),
            "Customer Success".to_string(),
        ],
        benefits: vec![
            "Remote-first culture".to_string(),
            "Competitive compensation".to_string(),
            "Health & wellness benefits".to_string(),
            "Professional development".to_string(),
        ],
    };

    let response = CareersResponse {
        jobs,
        company_info,
    };

    Ok(Json(response))
}
