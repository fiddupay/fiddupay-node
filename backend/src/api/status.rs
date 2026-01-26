use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    pub overall_status: String,
    pub services: Vec<ServiceStatus>,
    pub uptime_stats: UptimeStats,
    pub last_updated: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub description: String,
    pub status: String,
    pub response_time: Option<u32>,
    pub last_check: String,
}

#[derive(Serialize, Deserialize)]
pub struct UptimeStats {
    pub thirty_days: f64,
    pub ninety_days: f64,
    pub one_year: f64,
}

pub async fn get_system_status(
    State(_state): State<AppState>,
) -> Result<Json<SystemStatus>, StatusCode> {
    // In a real implementation, this would check actual services
    let services = vec![
        ServiceStatus {
            name: "Payment API".to_string(),
            description: "Core payment processing service".to_string(),
            status: "operational".to_string(),
            response_time: Some(45),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
        ServiceStatus {
            name: "Blockchain Monitoring".to_string(),
            description: "Transaction confirmation service".to_string(),
            status: "operational".to_string(),
            response_time: Some(120),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
        ServiceStatus {
            name: "Webhook Delivery".to_string(),
            description: "Real-time notification system".to_string(),
            status: "operational".to_string(),
            response_time: Some(30),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
        ServiceStatus {
            name: "Dashboard".to_string(),
            description: "Merchant dashboard interface".to_string(),
            status: "operational".to_string(),
            response_time: Some(25),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
    ];

    let uptime_stats = UptimeStats {
        thirty_days: 99.99,
        ninety_days: 99.98,
        one_year: 99.97,
    };

    let overall_status = if services.iter().all(|s| s.status == "operational") {
        "operational".to_string()
    } else {
        "degraded".to_string()
    };

    let status = SystemStatus {
        overall_status,
        services,
        uptime_stats,
        last_updated: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(status))
}
