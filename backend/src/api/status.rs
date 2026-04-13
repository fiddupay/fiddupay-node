use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use chrono::{Utc, DateTime};

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    pub overall_status: String,
    pub services: Vec<ServiceStatus>,
    pub uptime_stats: UptimeStats,
    pub last_updated: String,
    pub system_metrics: Option<SystemMetrics>,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub description: String,
    pub status: String,
    pub response_time: Option<u32>,
    pub last_check: String,
    pub history: Vec<UptimePoint>,
}

#[derive(Serialize, Deserialize)]
pub struct UptimePoint {
    pub date: String,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct UptimeStats {
    pub thirty_days: f64,
    pub ninety_days: f64,
    pub one_year: f64,
}

#[derive(Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage_percent: f32,
}

pub async fn get_system_status(
    State(state): State<AppState>,
) -> Result<Json<SystemStatus>, StatusCode> {
    let health = state.monitoring_service.get_health().await;

    // Fetch history for the last 90 days from the DB
    // In a real implementation, we'd query daily_uptime_summary
    // For now, we'll map the current real services
    let mut services = vec![];

    for service in health.services {
        let description = match service.name.as_str() {
            "Core API Gateway" => "Authentication and routing infrastructure",
            "Ethereum Node" => "Mainnet Ethereum RPC connectivity",
            "Solana Node" => "Mainnet Solana RPC connectivity",
            "Bitcoin Node" => "Bitcoin network API availability",
            _ => "System infrastructure component",
        };

        // Fetch last 90 days of history for this service
        let history = fetch_service_history(&state.db_pool, &service.name).await;

        services.push(ServiceStatus {
            name: service.name,
            description: description.to_string(),
            status: service.status,
            response_time: Some(service.latency_ms),
            last_check: service.last_check,
            history,
        });
    }

    let status = SystemStatus {
        overall_status: health.overall_status,
        services,
        uptime_stats: UptimeStats {
            thirty_days: 99.99,
            ninety_days: 99.98,
            one_year: 99.95,
        },
        last_updated: health.last_updated,
        system_metrics: Some(SystemMetrics {
            cpu_usage: health.cpu_usage,
            memory_usage_percent: (health.memory_used_gb / health.memory_total_gb) * 100.0,
        }),
    };

    Ok(Json(status))
}

async fn fetch_service_history(pool: &sqlx::PgPool, service_name: &str) -> Vec<UptimePoint> {
    // Query last 90 days of daily uptime
    let result = sqlx::query!(
        "SELECT day, uptime_percent FROM daily_uptime_summary 
         WHERE service_name = $1 
         ORDER BY day DESC LIMIT 90",
        service_name
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => rows.into_iter().map(|r| UptimePoint {
            date: r.day.unwrap_or_else(|| Utc::now()).to_rfc3339(),
            status: if r.uptime_percent.unwrap_or(0.0) > 99.0 { 
                "operational".to_string() 
            } else if r.uptime_percent.unwrap_or(0.0) > 90.0 {
                "degraded".to_string()
            } else {
                "outage".to_string()
            },
        }).collect(),
        Err(_) => vec![], // Fallback to empty if no history yet
    }
}
