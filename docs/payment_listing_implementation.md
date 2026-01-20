# Payment Listing and Filtering Implementation

## Overview
This document describes the implementation of task 6.7: Payment listing and filtering functionality for the crypto payment gateway service.

## Requirements
**Validates: Requirements 11.3** - Support filtering analytics by date range, blockchain, and payment status

## Implementation Details

### 1. Data Models (src/payment/models.rs)

#### PaymentFilters
A comprehensive filter structure for querying payments:
```rust
pub struct PaymentFilters {
    pub status: Option<PaymentStatus>,           // Filter by payment status
    pub blockchain: Option<String>,              // Filter by network (e.g., "SOLANA", "BEP20")
    pub from_date: Option<DateTime<Utc>>,        // Start of date range
    pub to_date: Option<DateTime<Utc>>,          // End of date range
    pub page: Option<u32>,                       // Page number (default: 1)
    pub page_size: Option<u32>,                  // Items per page (default: 20, max: 100)
}
```

**Features:**
- Default values: page=1, page_size=20
- Automatic validation: page_size capped at 100
- All filters are optional for maximum flexibility

#### PaymentList
Paginated response structure:
```rust
pub struct PaymentList {
    pub payments: Vec<PaymentResponse>,  // List of payment responses
    pub total: i64,                      // Total count matching filters
    pub page: u32,                       // Current page number
    pub page_size: u32,                  // Items per page
    pub total_pages: u32,                // Total pages available
}
```

### 2. Service Implementation (src/services/payment_service.rs)

#### Main Function: list_payments
```rust
pub async fn list_payments(
    &self,
    merchant_id: i64,
    filters: PaymentFilters,
) -> Result<PaymentList, PaymentServiceError>
```

**Features:**
1. **Pagination**
   - Validates and enforces page >= 1
   - Caps page_size at 100 items
   - Calculates offset for database query
   - Returns total_pages for UI pagination

2. **Status Filter**
   - Filters by PaymentStatus enum
   - Converts enum to database string format
   - Supports: PENDING, CONFIRMING, CONFIRMED, FAILED, REFUNDED

3. **Blockchain Filter**
   - Filters by network field
   - Supports: SOLANA, BEP20, ARBITRUM, POLYGON, SOLANA_SPL

4. **Date Range Filter**
   - Filters by created_at timestamp
   - Supports from_date (>=) and to_date (<=)
   - Both boundaries are inclusive

5. **Ordering**
   - Results ordered by created_at DESC (newest first)
   - Consistent ordering for pagination

#### Helper Functions

**count_payments**
- Counts total payments matching filters
- Used for pagination metadata
- Mirrors filter logic from main query

**convert_to_response**
- Converts PaymentTransaction to PaymentResponse
- Parses crypto_type and status strings to enums
- Loads partial payment info if enabled
- Generates payment_link and qr_code_data

**get_partial_payments**
- Loads partial payment records for a payment
- Ordered by created_at ASC
- Returns empty vec if none exist

**parse_crypto_type / parse_status**
- Converts database strings to enums
- Provides fallback defaults for unknown values

### 3. Error Handling

```rust
pub enum PaymentServiceError {
    DatabaseError(sqlx::Error),
    PaymentNotFound,
    InvalidFilters(String),
}
```

- Database errors propagated from sqlx
- Type-safe error handling with thiserror
- Clear error messages for debugging

### 4. Query Building

The implementation uses dynamic SQL query building:
1. Base query filters by merchant_id
2. Conditionally adds WHERE clauses for each filter
3. Adds ORDER BY and pagination (LIMIT/OFFSET)
4. Uses parameterized queries to prevent SQL injection
5. Binds parameters in correct order

Example generated query:
```sql
SELECT * FROM payment_transactions 
WHERE merchant_id = $1 
  AND status = $2 
  AND network = $3 
  AND created_at >= $4 
  AND created_at <= $5 
ORDER BY created_at DESC 
LIMIT $6 OFFSET $7
```

## Testing

Comprehensive unit tests in `tests/payment_listing_tests.rs`:

1. **test_list_payments_default_pagination** - Default behavior
2. **test_list_payments_with_status_filter** - Status filtering
3. **test_list_payments_with_blockchain_filter** - Blockchain filtering
4. **test_list_payments_with_date_range_filter** - Date range filtering
5. **test_list_payments_with_pagination** - Multi-page results
6. **test_list_payments_with_combined_filters** - Multiple filters together
7. **test_list_payments_max_page_size** - Page size capping
8. **test_list_payments_empty_result** - No results case
9. **test_list_payments_ordering** - Result ordering verification
10. **test_list_payments_different_merchants** - Merchant isolation

## Usage Examples

### Basic Listing
```rust
let service = PaymentService::new(db_pool);
let filters = PaymentFilters::default();
let result = service.list_payments(merchant_id, filters).await?;
```

### Filter by Status
```rust
let filters = PaymentFilters {
    status: Some(PaymentStatus::Confirmed),
    ..Default::default()
};
let result = service.list_payments(merchant_id, filters).await?;
```

### Filter by Blockchain
```rust
let filters = PaymentFilters {
    blockchain: Some("SOLANA".to_string()),
    ..Default::default()
};
let result = service.list_payments(merchant_id, filters).await?;
```

### Date Range with Pagination
```rust
let filters = PaymentFilters {
    from_date: Some(start_date),
    to_date: Some(end_date),
    page: Some(2),
    page_size: Some(50),
    ..Default::default()
};
let result = service.list_payments(merchant_id, filters).await?;
```

### Combined Filters
```rust
let filters = PaymentFilters {
    status: Some(PaymentStatus::Confirmed),
    blockchain: Some("BEP20".to_string()),
    from_date: Some(last_week),
    to_date: Some(now),
    page: Some(1),
    page_size: Some(20),
};
let result = service.list_payments(merchant_id, filters).await?;
```

## API Integration

This service method will be exposed via REST API endpoint:
```
GET /api/v1/payments?status=CONFIRMED&blockchain=SOLANA&from=2024-01-01&to=2024-12-31&page=1&page_size=20
```

Response:
```json
{
  "payments": [...],
  "total": 150,
  "page": 1,
  "page_size": 20,
  "total_pages": 8
}
```

## Performance Considerations

1. **Database Indexes**
   - Existing index on merchant_id
   - Consider composite index: (merchant_id, status, created_at)
   - Consider index on network for blockchain filtering

2. **Query Optimization**
   - Uses LIMIT/OFFSET for pagination
   - Separate COUNT query for total (could be optimized with window functions)
   - Parameterized queries for query plan caching

3. **Pagination Limits**
   - Max page_size of 100 prevents excessive memory usage
   - Offset-based pagination works well for moderate datasets
   - Consider cursor-based pagination for very large datasets

## Future Enhancements

1. **Additional Filters**
   - Filter by amount range
   - Filter by crypto_type
   - Full-text search on description/metadata

2. **Sorting Options**
   - Sort by amount, status, blockchain
   - Ascending/descending order

3. **Performance**
   - Cursor-based pagination for large datasets
   - Caching for frequently accessed pages
   - Materialized views for analytics

4. **Export**
   - CSV export of filtered results
   - Streaming for large exports

## Compliance

This implementation satisfies:
- ✅ Requirement 11.3: Support filtering analytics by date range, blockchain, and payment status
- ✅ Pagination support for large result sets
- ✅ Merchant isolation (only returns payments for specified merchant)
- ✅ Type-safe error handling
- ✅ SQL injection prevention via parameterized queries
- ✅ Comprehensive test coverage
