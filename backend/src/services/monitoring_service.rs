use crate::config::Config;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: String,
    pub cpu_usage: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub db_connected: bool,
    pub redis_connected: bool,
    pub services: Vec<ServiceHealth>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub name: String,
    pub status: String, // operational, degraded, outage
    pub latency_ms: u32,
    pub last_check: String,
}

pub struct MonitoringService {
    db_pool: PgPool,
    config: Config,
    redis_client: redis::Client,
    current_health: Arc<RwLock<SystemHealth>>,
}

impl MonitoringService {
    pub fn new(db_pool: PgPool, config: Config, redis_client: redis::Client) -> Self {
        let initial_health = SystemHealth {
            overall_status: "initializing".to_string(),
            cpu_usage: 0.0,
            memory_used_gb: 0.0,
            memory_total_gb: 0.0,
            db_connected: false,
            redis_connected: false,
            services: vec![],
            last_updated: Utc::now().to_rfc3339(),
        };

        Self {
            db_pool,
            config,
            redis_client,
            current_health: Arc::new(RwLock::new(initial_health)),
        }
    }

    pub async fn get_health(&self) -> SystemHealth {
        self.current_health.read().await.clone()
    }

    pub fn start_polling(self: Arc<Self>) {
        let service = self.clone();
        tokio::spawn(async move {
            let mut sys = System::new_all();
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                service.perform_checks(&mut sys).await;
            }
        });
    }

    async fn perform_checks(&self, sys: &mut System) {
        // 1. System Metrics
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_used = sys.used_memory() as f32 / 1024.0 / 1024.0 / 1024.0;
        let memory_total = sys.total_memory() as f32 / 1024.0 / 1024.0 / 1024.0;

        // 2. DB Check
        let db_connected = sqlx::query("SELECT 1")
            .fetch_one(&self.db_pool)
            .await
            .is_ok();

        // 3. Redis Check
        let redis_connected = match self.redis_client.get_multiplexed_async_connection().await {
            Ok(mut conn) => redis::cmd("PING")
                .query_async::<String>(&mut conn)
                .await
                .is_ok(),
            Err(_) => false,
        };

        // 4. RPC Probes
        let mut services = vec![];

        // Define all supported nodes and their status
        let probe_configs = [
            (
                "Ethereum Node",
                self.config.ethereum_enabled,
                self.config.ethereum_rpc_url.clone(),
                false,
            ),
            (
                "Solana Node",
                self.config.solana_enabled,
                self.config.solana_rpc_url.clone(),
                false,
            ),
            (
                "Bitcoin Node",
                self.config.bitcoin_enabled,
                self.config.bitcoin_rpc_url.clone(),
                true,
            ),
            (
                "BNB Node",
                self.config.bsc_enabled,
                self.config.bsc_rpc_url.clone(),
                false,
            ),
            (
                "Polygon Node",
                self.config.polygon_enabled,
                self.config.polygon_rpc_url.clone(),
                false,
            ),
            (
                "Arbitrum Node",
                self.config.arbitrum_enabled,
                self.config.arbitrum_rpc_url.clone(),
                false,
            ),
        ];

        for (name, enabled, url, is_btc) in probe_configs {
            if enabled {
                let probe_url = if is_btc {
                    // Try to detect if it's a standard RPC or Esplora-style API
                    if url.contains("blockstream.info") || url.contains("mempool.space") {
                        format!("{}/blocks/tip/height", url.trim_end_matches('/'))
                    } else {
                        url.to_string()
                    }
                } else {
                    url.to_string()
                };
                services.push(self.probe_rpc(name, &probe_url).await);
            } else {
                services.push(ServiceHealth {
                    name: name.to_string(),
                    status: "disabled".to_string(),
                    latency_ms: 0,
                    last_check: Utc::now().to_rfc3339(),
                });
            }
        }

        // Core Dashboard Service (Self check)
        services.push(ServiceHealth {
            name: "Core API Gateway".to_string(),
            status: if db_connected {
                "operational".to_string()
            } else {
                "outage".to_string()
            },
            latency_ms: 0,
            last_check: Utc::now().to_rfc3339(),
        });

        // Determine overall status
        // IMPORTANT: Only count "outage" for services that were actually ENABLED but failing
        let mut platform_failure = !db_connected;
        let mut platform_degraded = false;

        // Check enabled services for actual issues
        let enabled_services = services.iter().filter(|s| match s.name.as_str() {
            "Ethereum Node" => self.config.ethereum_enabled,
            "Solana Node" => self.config.solana_enabled,
            "Bitcoin Node" => self.config.bitcoin_enabled,
            "BNB Node" => self.config.bsc_enabled,
            "Polygon Node" => self.config.polygon_enabled,
            "Arbitrum Node" => self.config.arbitrum_enabled,
            "Core API Gateway" => true,
            _ => false,
        });

        for s in enabled_services {
            if s.status == "outage" {
                platform_failure = true;
            } else if s.status == "degraded" {
                platform_degraded = true;
            }
        }

        let overall_status = if platform_failure {
            "outage".to_string()
        } else if platform_degraded {
            "degraded".to_string()
        } else {
            "operational".to_string()
        };

        let health = SystemHealth {
            overall_status,
            cpu_usage,
            memory_used_gb: memory_used,
            memory_total_gb: memory_total,
            db_connected,
            redis_connected,
            services: services.clone(),
            last_updated: Utc::now().to_rfc3339(),
        };

        // Update cache
        {
            let mut current = self.current_health.write().await;
            *current = health.clone();
        }

        // Persist history (once every 10 minutes to avoid DB bloat)
        let now = Utc::now();
        if now.minute().is_multiple_of(10) {
            for service in services {
                let _ = sqlx::query(
                    "INSERT INTO system_health_history (service_name, status, latency_ms, cpu_usage, memory_usage_gb) 
                     VALUES ($1, $2, $3, $4, $5)"
                )
                .bind(&service.name)
                .bind(&service.status)
                .bind(service.latency_ms as i32)
                .bind(cpu_usage)
                .bind(memory_used)
                .execute(&self.db_pool)
                .await;
            }
        }
    }

    async fn probe_rpc(&self, name: &str, url: &str) -> ServiceHealth {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let start = std::time::Instant::now();
        let result = client.get(url).send().await;
        let latency = start.elapsed().as_millis() as u32;

        let status = match result {
            Ok(resp)
                if resp.status().is_success()
                    || resp.status().as_u16() == 405
                    || resp.status().as_u16() == 401 =>
            {
                // Many RPCs return 405 Method Not Allowed on a naked GET, but they are UP
                if latency > 1000 {
                    "degraded".to_string()
                } else {
                    "operational".to_string()
                }
            }
            _ => "outage".to_string(),
        };

        ServiceHealth {
            name: name.to_string(),
            status,
            latency_ms: latency,
            last_check: Utc::now().to_rfc3339(),
        }
    }
}

// Extension trait for minute checking
trait MinuteExt {
    fn minute(&self) -> u32;
}

impl MinuteExt for chrono::DateTime<Utc> {
    fn minute(&self) -> u32 {
        use chrono::Timelike;
        self.time().minute()
    }
}
