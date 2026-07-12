// Delora Models
// All Delora API request/response types and internal application types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Delora API Request Types ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct QuoteRequest {
    pub sender_address: String,
    pub receiver_address: String,
    pub origin_chain_id: u64,
    pub destination_chain_id: u64,
    pub amount: String,
    pub origin_currency: String,
    pub destination_currency: String,
    pub integrator: String,
    pub fee: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_bridges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_bridges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_exchanges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_exchanges: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdvancedRoutesRequest {
    pub sender_address: String,
    pub receiver_address: String,
    pub origin_chain_id: u64,
    pub destination_chain_id: u64,
    pub amount: String,
    pub origin_currency: String,
    pub destination_currency: String,
    pub integrator: String,
    pub fee: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_bridges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_bridges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_exchanges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_exchanges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_routes: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepTransactionRequest {
    pub step: RouteStep,
    pub context: StepTransactionContext,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepTransactionContext {
    pub sender_address: String,
    pub receiver_address: String,
}

// ── Delora API Response Types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub input_amount: String,
    pub output_amount: String,
    #[serde(default)]
    pub min_output_amount: Option<String>,
    pub adapter: String,
    pub calldata: Calldata,
    pub fees: FeeInfo,
    #[serde(default)]
    pub gas: Option<GasInfo>,
    #[serde(default)]
    pub warnings: Vec<DeloraWarning>,
    #[serde(default)]
    pub approval_address: Option<String>,
    #[serde(default)]
    pub estimated_time_sec: Option<u64>,
    #[serde(default)]
    pub bridge_scan: Option<serde_json::Value>,
    #[serde(default)]
    pub transaction_size: Option<serde_json::Value>,
    #[serde(default)]
    pub usd: Option<UsdPrices>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calldata {
    pub to: String,
    pub value: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeInfo {
    pub total: FeeItem,
    #[serde(default)]
    pub breakdown: Vec<FeeItem>,
    #[serde(default)]
    pub total_usd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeItem {
    pub amount: String,
    pub currency_symbol: String,
    #[serde(default)]
    pub currency_address: Option<String>,
    pub chain_id: u64,
    #[serde(default)]
    pub decimals: Option<u32>,
    #[serde(rename = "type", default)]
    pub fee_type: Option<String>,
    #[serde(default)]
    pub amount_usd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasInfo {
    #[serde(default)]
    pub gas_price: Option<String>,
    #[serde(default)]
    pub gas_limit: Option<String>,
    #[serde(default)]
    pub max_fee_per_gas: Option<String>,
    #[serde(default)]
    pub max_priority_fee_per_gas: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeloraWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsdPrices {
    #[serde(default)]
    pub origin_amount_usd: Option<String>,
    #[serde(default)]
    pub destination_amount_usd: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdvancedRoutesResponse {
    pub routes: Vec<AdvancedRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedRoute {
    pub id: String,
    pub input_amount: String,
    #[serde(default)]
    pub output_amount: Option<String>,
    #[serde(default)]
    pub min_output_amount: Option<String>,
    #[serde(default)]
    pub fees: Option<FeeInfo>,
    pub adapter: String,
    pub is_multistep: bool,
    pub steps: Vec<RouteStep>,
    #[serde(default)]
    pub estimated_time_sec: Option<u64>,
    #[serde(default)]
    pub warnings: Option<Vec<DeloraWarning>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStep {
    pub id: String,
    pub route_id: String,
    #[serde(rename = "type")]
    pub step_type: String,
    pub tool: String,
    pub action: StepAction,
    pub estimate: StepEstimate,
    #[serde(default)]
    pub execution: Option<serde_json::Value>,
    #[serde(default)]
    pub transaction_request: Option<TransactionRequest>,
    #[serde(default)]
    pub integrator: Option<String>,
    #[serde(default)]
    pub fee: Option<f64>,
    #[serde(default)]
    pub warnings: Option<Vec<DeloraWarning>>,
    #[serde(default)]
    pub bridge_scan: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepAction {
    pub from_chain_id: u64,
    pub to_chain_id: u64,
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub from_amount: String,
    #[serde(default)]
    pub slippage: Option<f64>,
    #[serde(default)]
    pub from_address: Option<String>,
    #[serde(default)]
    pub to_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub decimals: u32,
    pub chain_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEstimate {
    pub from_amount: String,
    pub to_amount: String,
    #[serde(default)]
    pub to_amount_min: Option<String>,
    #[serde(default)]
    pub fees: Option<FeeInfo>,
    #[serde(default)]
    pub approval_address: Option<String>,
    #[serde(default)]
    pub estimated_time_sec: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub to: String,
    pub value: String,
    pub data: String,
    #[serde(default)]
    pub gas: Option<GasInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub chain_id: u64,
    pub name: String,
    #[serde(default)]
    pub native_currency: Option<NativeCurrencyInfo>,
    #[serde(default)]
    pub rpc_urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeCurrencyInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenListResponse {
    pub tokens: HashMap<String, Vec<TokenItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenItem {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u32,
    pub chain_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub key: String,
    pub name: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResponse {
    pub prices: HashMap<String, serde_json::Value>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeloraApiError {
    pub code: String,
    pub message: String,
}

// ── Internal Application Types ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CrossChainQuoteResponse {
    pub quote_id: String,
    pub origin_chain_id: u64,
    pub destination_chain_id: u64,
    pub origin_currency: CurrencySummary,
    pub destination_currency: CurrencySummary,
    pub input_amount: String,
    pub input_amount_display: String,
    pub output_amount: String,
    pub output_amount_display: String,
    pub min_output_amount: String,
    pub fees: QuoteFeeBreakdown,
    pub estimated_time_sec: Option<u64>,
    pub calldata: Calldata,
    pub gas: Option<GasInfo>,
    pub approval_address: Option<String>,
    pub warnings: Vec<DeloraWarning>,
    pub expires_at: String,
    pub route: Option<AdvancedRouteSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdvancedRouteSummary {
    pub route_id: String,
    pub adapter: String,
    pub is_multistep: bool,
    pub steps_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrencySummary {
    pub symbol: String,
    pub name: String,
    pub address: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub decimals: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuoteFeeBreakdown {
    pub delora_fee: String,
    pub delora_fee_usd: String,
    pub integrator_fee: String,
    pub integrator_fee_usd: String,
    pub gas_fee_estimate: String,
    pub gas_fee_estimate_usd: String,
    pub total_fee: String,
    pub total_fee_usd: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum CrossChainPaymentStatus {
    #[sqlx(rename = "quote_requested")]
    #[serde(rename = "quote_requested")]
    QuoteRequested,
    #[sqlx(rename = "tx_submitted")]
    #[serde(rename = "tx_submitted")]
    TxSubmitted,
    #[sqlx(rename = "tx_confirmed")]
    #[serde(rename = "tx_confirmed")]
    TxConfirmed,
    #[sqlx(rename = "bridge_pending")]
    #[serde(rename = "bridge_pending")]
    BridgePending,
    #[sqlx(rename = "bridge_complete")]
    #[serde(rename = "bridge_complete")]
    BridgeComplete,
    #[sqlx(rename = "completed")]
    #[serde(rename = "completed")]
    Completed,
    #[sqlx(rename = "failed")]
    #[serde(rename = "failed")]
    Failed,
    #[sqlx(rename = "expired")]
    #[serde(rename = "expired")]
    Expired,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossChainStatusResponse {
    pub payment_id: String,
    pub status: CrossChainPaymentStatus,
    pub origin_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub bridge_scan_url: Option<String>,
    pub confirmations: Option<i32>,
    pub estimated_completion_sec: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainSummary {
    pub chain_id: u64,
    pub name: String,
    pub native_symbol: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenSummary {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u32,
    pub chain_id: u64,
}

// ── Database Row Type ────────────────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CrossChainPaymentRow {
    pub id: i64,
    pub quote_id: uuid::Uuid,
    pub payment_transaction_id: Option<i64>,
    pub merchant_id: i64,
    pub invoice_id: Option<uuid::Uuid>,
    pub origin_chain_id: i64,
    pub origin_currency_address: String,
    pub origin_currency_symbol: String,
    pub origin_currency_decimals: i32,
    pub destination_chain_id: i64,
    pub destination_currency_address: String,
    pub destination_currency_symbol: String,
    pub destination_currency_decimals: i32,
    pub input_amount: String,
    pub output_amount: String,
    pub min_output_amount: String,
    pub delora_fee_amount: Option<String>,
    pub delora_fee_usd: Option<String>,
    pub integrator_fee_amount: Option<String>,
    pub integrator_fee_usd: Option<String>,
    pub integrator_fee_rate: Option<rust_decimal::Decimal>,
    pub adapter: String,
    pub route_id: Option<String>,
    pub route_snapshot: Option<serde_json::Value>,
    pub is_multistep: bool,
    pub is_advanced: bool,
    pub status: CrossChainPaymentStatus,
    pub sender_address: Option<String>,
    pub merchant_destination_address: String,
    pub calldata: serde_json::Value,
    pub calldata_to: String,
    pub approval_address: Option<String>,
    pub origin_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub origin_block_number: Option<i64>,
    pub destination_block_number: Option<i64>,
    pub origin_confirmations: Option<i32>,
    pub bridge_scan_metadata: Option<serde_json::Value>,
    pub quote_expires_at: DateTime<Utc>,
    pub tx_submitted_at: Option<DateTime<Utc>>,
    pub origin_confirmed_at: Option<DateTime<Utc>>,
    pub bridge_completed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_reason: Option<String>,
    pub delora_warnings: Option<serde_json::Value>,
    pub estimated_time_sec: Option<i64>,
    pub gas_info: Option<serde_json::Value>,
    pub sandbox_mode: bool,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
