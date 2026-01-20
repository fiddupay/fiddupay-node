# Analytics Service Documentation

## Overview

The Analytics Service provides comprehensive reporting and data export capabilities for merchants to track their payment performance, transaction volumes, and fee payments across different blockchains.

## Features

### 1. Analytics Calculation

The `get_analytics` function provides detailed analytics for a specified date range with optional filtering:

**Parameters:**
- `merchant_id`: The merchant's unique identifier
- `start_date`: Start of the date range (inclusive)
- `end_date`: End of the date range (inclusive)
- `blockchain`: Optional filter by blockchain network (e.g., "SOLANA", "BEP20", "ARBITRUM", "POLYGON")
- `status`: Optional filter by payment status (e.g., "CONFIRMED", "PENDING", "FAILED")

**Returns:**
```rust
AnalyticsReport {
    total_volume_usd: Decimal,           // Total USD value of confirmed payments
    successful_payments: i64,             // Count of confirmed payments
    failed_payments: i64,                 // Count of failed/expired payments
    total_fees_paid: Decimal,             // Total fees collected in USD
    average_transaction_value: Decimal,   // Average value per successful payment
    by_blockchain: HashMap<String, BlockchainStats>, // Stats per blockchain
}
```

**Blockchain Stats:**
```rust
BlockchainStats {
    volume_usd: Decimal,      // Total volume for this blockchain
    payment_count: i64,       // Number of payments on this blockchain
    average_value: Decimal,   // Average payment value for this blockchain
}
```

### 2. CSV Export

The `export_csv` function exports detailed payment data in CSV format for external analysis:

**Parameters:**
- `merchant_id`: The merchant's unique identifier
- `start_date`: Start of the date range (inclusive)
- `end_date`: End of the date range (inclusive)
- `blockchain`: Optional filter by blockchain network
- `status`: Optional filter by payment status

**Returns:**
A CSV string with the following columns:
- Payment ID
- Status
- Amount (in crypto)
- Amount USD
- Crypto Type
- Network
- Transaction Hash
- From Address
- To Address
- Fee Percentage
- Fee Amount (in crypto)
- Fee Amount USD
- Description
- Created At (RFC3339 format)
- Confirmed At (RFC3339 format)
- Expires At (RFC3339 format)

**CSV Features:**
- Proper escaping of fields containing commas, quotes, or newlines
- RFC3339 timestamp format for easy parsing
- Empty fields for optional data (e.g., transaction_hash for pending payments)
- Ordered by creation date (most recent first)

## Usage Examples

### Example 1: Get Overall Analytics

```rust
let analytics_service = AnalyticsService::new(db_pool);

let start_date = Utc::now() - Duration::days(30);
let end_date = Utc::now();

let report = analytics_service
    .get_analytics(merchant_id, start_date, end_date, None, None)
    .await?;

println!("Total Volume: ${}", report.total_volume_usd);
println!("Successful Payments: {}", report.successful_payments);
println!("Failed Payments: {}", report.failed_payments);
println!("Total Fees: ${}", report.total_fees_paid);
println!("Average Transaction: ${}", report.average_transaction_value);

for (network, stats) in report.by_blockchain {
    println!("{}: ${} ({} payments)", network, stats.volume_usd, stats.payment_count);
}
```

### Example 2: Get Analytics for Specific Blockchain

```rust
let report = analytics_service
    .get_analytics(
        merchant_id,
        start_date,
        end_date,
        Some("SOLANA".to_string()),
        None
    )
    .await?;

println!("Solana Volume: ${}", report.total_volume_usd);
```

### Example 3: Get Analytics for Confirmed Payments Only

```rust
let report = analytics_service
    .get_analytics(
        merchant_id,
        start_date,
        end_date,
        None,
        Some("CONFIRMED".to_string())
    )
    .await?;

println!("Confirmed Payments: {}", report.successful_payments);
```

### Example 4: Export CSV

```rust
let csv_data = analytics_service
    .export_csv(
        merchant_id,
        start_date,
        end_date,
        None,
        None
    )
    .await?;

// Save to file
std::fs::write("payments_export.csv", csv_data)?;

// Or return as HTTP response
Ok(Response::builder()
    .header("Content-Type", "text/csv")
    .header("Content-Disposition", "attachment; filename=payments.csv")
    .body(csv_data)?)
```

### Example 5: Export Filtered CSV

```rust
// Export only confirmed Solana payments
let csv_data = analytics_service
    .export_csv(
        merchant_id,
        start_date,
        end_date,
        Some("SOLANA".to_string()),
        Some("CONFIRMED".to_string())
    )
    .await?;
```

## Database Queries

### Analytics Query

The analytics calculation uses aggregation queries for efficiency:

```sql
SELECT 
    COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN amount_usd ELSE 0 END), 0) as total_volume_usd,
    COUNT(CASE WHEN status = 'CONFIRMED' THEN 1 END) as successful_payments,
    COUNT(CASE WHEN status IN ('FAILED', 'EXPIRED') THEN 1 END) as failed_payments,
    COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN fee_amount_usd ELSE 0 END), 0) as total_fees_paid
FROM payment_transactions
WHERE merchant_id = $1
    AND created_at >= $2
    AND created_at <= $3
```

### Blockchain Stats Query

```sql
SELECT 
    network,
    COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN amount_usd ELSE 0 END), 0) as volume_usd,
    COUNT(CASE WHEN status = 'CONFIRMED' THEN 1 END) as payment_count
FROM payment_transactions
WHERE merchant_id = $1
    AND created_at >= $2
    AND created_at <= $3
GROUP BY network
```

## Performance Considerations

1. **Indexes**: Ensure the following indexes exist for optimal performance:
   - `idx_payment_transactions_merchant` on `merchant_id`
   - `idx_payment_transactions_created` on `created_at`
   - `idx_payment_transactions_status` on `status`
   - Composite index on `(merchant_id, created_at, status)`

2. **Date Range**: Limit date ranges to reasonable periods (e.g., 90 days) to prevent slow queries

3. **CSV Export**: For large datasets, consider:
   - Pagination or streaming for exports
   - Background job processing for very large exports
   - Caching frequently requested exports

4. **Caching**: Consider caching analytics results for:
   - Fixed date ranges (e.g., last month)
   - Frequently accessed reports
   - Cache invalidation on new payment confirmations

## Error Handling

The service returns `ServiceError` for various failure scenarios:

- `ServiceError::Database`: Database query failures
- `ServiceError::MerchantNotFound`: Invalid merchant_id
- `ServiceError::Internal`: Unexpected errors

## Testing

The analytics service includes comprehensive tests:

1. **Unit Tests**: Test calculation logic, CSV escaping, edge cases
2. **Integration Tests**: Test with real database queries
3. **Property Tests**: Verify correctness properties (see tasks.md)

## API Integration

The analytics service is typically exposed via REST API endpoints:

```
GET /api/v1/analytics
Query Parameters:
  - from: ISO8601 date (required)
  - to: ISO8601 date (required)
  - blockchain: string (optional)
  - status: string (optional)

GET /api/v1/analytics/export
Query Parameters:
  - from: ISO8601 date (required)
  - to: ISO8601 date (required)
  - blockchain: string (optional)
  - status: string (optional)
Response: text/csv
```

## Future Enhancements

Potential improvements for the analytics service:

1. **Real-time Analytics**: WebSocket updates for live metrics
2. **Custom Reports**: User-defined report templates
3. **Comparative Analytics**: Period-over-period comparisons
4. **Visualization Data**: Pre-calculated data for charts/graphs
5. **Export Formats**: Support for JSON, Excel, PDF exports
6. **Scheduled Reports**: Automated email reports
7. **Advanced Filtering**: Multiple status filters, amount ranges
8. **Aggregation Levels**: Daily, weekly, monthly rollups
