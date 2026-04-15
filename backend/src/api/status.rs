use crate::api::state::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    pub overall_status: String,
    pub services: Vec<ServiceStatus>,
    pub uptime_stats: UptimeStats,
    pub last_updated: String,
    pub system_metrics: Option<SystemMetrics>,
    pub past_incidents: Vec<SystemIncident>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemIncident {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub status: String,
    pub severity: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
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

    // 1. Map current services
    let mut services = vec![];
    for service in health.services {
        let description = match service.name.as_str() {
            "Core API Gateway" => "Authentication and routing infrastructure",
            "Ethereum Node" => "Mainnet Ethereum RPC connectivity",
            "Solana Node" => "Mainnet Solana RPC connectivity",
            "Bitcoin Node" => "Bitcoin network API availability",
            _ => "System infrastructure component",
        };

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

    // 2. Fetch Aggregate Uptime Stats
    let uptime_stats = fetch_aggregate_uptime(&state.db_pool).await;

    // 3. Fetch Recent Incidents
    let past_incidents = fetch_recent_incidents(&state.db_pool).await;

    let status = SystemStatus {
        overall_status: health.overall_status,
        services,
        uptime_stats,
        last_updated: health.last_updated,
        system_metrics: Some(SystemMetrics {
            cpu_usage: health.cpu_usage,
            memory_usage_percent: (health.memory_used_gb / health.memory_total_gb) * 100.0,
        }),
        past_incidents,
    };

    Ok(Json(status))
}

async fn fetch_service_history(pool: &sqlx::PgPool, service_name: &str) -> Vec<UptimePoint> {
    let result = sqlx::query(
        "SELECT day, uptime_percent FROM daily_uptime_summary 
         WHERE service_name = $1 
         ORDER BY day DESC LIMIT 90",
    )
    .bind(service_name)
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => rows
            .into_iter()
            .map(|r| {
                let day: Option<DateTime<Utc>> = r.get("day");
                let uptime_percent: Option<f64> = r.get("uptime_percent");
                UptimePoint {
                    date: day.unwrap_or_else(|| Utc::now()).to_rfc3339(),
                    status: if uptime_percent.unwrap_or(0.0) >= 99.0 {
                        "operational".to_string()
                    } else if uptime_percent.unwrap_or(0.0) >= 90.0 {
                        "degraded".to_string()
                    } else {
                        "outage".to_string()
                    },
                }
            })
            .collect(),
        Err(_) => vec![],
    }
}

async fn fetch_aggregate_uptime(pool: &sqlx::PgPool) -> UptimeStats {
    let stats = sqlx::query(
        r#"
        SELECT 
            AVG(uptime_percent) FILTER (WHERE day >= NOW() - INTERVAL '30 days') as thirty,
            AVG(uptime_percent) FILTER (WHERE day >= NOW() - INTERVAL '90 days') as ninety,
            AVG(uptime_percent) FILTER (WHERE day >= NOW() - INTERVAL '365 days') as yearly
        FROM daily_uptime_summary
        "#
    )
    .fetch_one(pool)
    .await;

    match stats {
        Ok(row) => UptimeStats {
            thirty_days: row.get::<Option<f64>, _>("thirty").unwrap_or(100.0).round(),
            ninety_days: row.get::<Option<f64>, _>("ninety").unwrap_or(100.0).round(),
            one_year: row.get::<Option<f64>, _>("yearly").unwrap_or(100.0).round(),
        },
        Err(_) => UptimeStats {
            thirty_days: 100.0,
            ninety_days: 100.0,
            one_year: 100.0,
        },
    }
}

async fn fetch_recent_incidents(pool: &sqlx::PgPool) -> Vec<SystemIncident> {
    let result = sqlx::query(
        "SELECT id, title, description, status, severity, created_at, resolved_at FROM system_incidents 
         ORDER BY created_at DESC LIMIT 5"
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => rows
            .into_iter()
            .map(|r| SystemIncident {
                id: r.get("id"),
                title: r.get("title"),
                description: r.get("description"),
                status: r.get("status"),
                severity: r.get("severity"),
                created_at: r.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
                resolved_at: r.get::<Option<DateTime<Utc>>, _>("resolved_at").map(|d| d.to_rfc3339()),
            })
            .collect(),
        Err(_) => vec![],
    }
}
