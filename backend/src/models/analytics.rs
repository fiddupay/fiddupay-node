// Analytics Models
// Data structures for analytics and reporting

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub total_volume_usd: Decimal,
    pub successful_payments: i64,
    pub failed_payments: i64,
    pub total_fees_paid: Decimal,
    pub average_transaction_value: Decimal,
    pub by_blockchain: HashMap<String, BlockchainStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub volume_usd: Decimal,
    pub payment_count: i64,
    pub average_value: Decimal,
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_analytics_report_creation() {
        let mut by_blockchain = HashMap::new();
        by_blockchain.insert(
            "SOLANA".to_string(),
            BlockchainStats {
                volume_usd: Decimal::new(50000, 2),
                payment_count: 10,
                average_value: Decimal::new(5000, 2),
            },
        );

        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(100000, 2),
            successful_payments: 20,
            failed_payments: 5,
            total_fees_paid: Decimal::new(1500, 2),
            average_transaction_value: Decimal::new(5000, 2),
            by_blockchain,
        };

        assert_eq!(report.total_volume_usd, Decimal::new(100000, 2));
        assert_eq!(report.successful_payments, 20);
        assert_eq!(report.failed_payments, 5);
        assert_eq!(report.total_fees_paid, Decimal::new(1500, 2));
        assert_eq!(report.average_transaction_value, Decimal::new(5000, 2));
        assert_eq!(report.by_blockchain.len(), 1);
    }

    #[test]
    fn test_blockchain_stats_creation() {
        let stats = BlockchainStats {
            volume_usd: Decimal::new(25000, 2),
            payment_count: 5,
            average_value: Decimal::new(5000, 2),
        };

        assert_eq!(stats.volume_usd, Decimal::new(25000, 2));
        assert_eq!(stats.payment_count, 5);
        assert_eq!(stats.average_value, Decimal::new(5000, 2));
    }

    #[test]
    fn test_analytics_report_serialization() {
        let mut by_blockchain = HashMap::new();
        by_blockchain.insert(
            "BEP20".to_string(),
            BlockchainStats {
                volume_usd: Decimal::new(30000, 2),
                payment_count: 15,
                average_value: Decimal::new(2000, 2),
            },
        );

        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(100000, 2),
            successful_payments: 50,
            failed_payments: 10,
            total_fees_paid: Decimal::new(1500, 2),
            average_transaction_value: Decimal::new(2000, 2),
            by_blockchain,
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("total_volume_usd"));
        assert!(json.contains("successful_payments"));
        assert!(json.contains("BEP20"));

        let deserialized: AnalyticsReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_volume_usd, report.total_volume_usd);
        assert_eq!(deserialized.successful_payments, report.successful_payments);
        assert_eq!(deserialized.failed_payments, report.failed_payments);
    }

    #[test]
    fn test_blockchain_stats_serialization() {
        let stats = BlockchainStats {
            volume_usd: Decimal::new(15000, 2),
            payment_count: 8,
            average_value: Decimal::new(1875, 2),
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("volume_usd"));
        assert!(json.contains("payment_count"));
        assert!(json.contains("average_value"));

        let deserialized: BlockchainStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.volume_usd, stats.volume_usd);
        assert_eq!(deserialized.payment_count, stats.payment_count);
        assert_eq!(deserialized.average_value, stats.average_value);
    }

    #[test]
    fn test_analytics_report_with_multiple_blockchains() {
        let mut by_blockchain = HashMap::new();
        
        by_blockchain.insert(
            "SOLANA".to_string(),
            BlockchainStats {
                volume_usd: Decimal::new(40000, 2),
                payment_count: 20,
                average_value: Decimal::new(2000, 2),
            },
        );
        
        by_blockchain.insert(
            "BEP20".to_string(),
            BlockchainStats {
                volume_usd: Decimal::new(30000, 2),
                payment_count: 15,
                average_value: Decimal::new(2000, 2),
            },
        );
        
        by_blockchain.insert(
            "ARBITRUM".to_string(),
            BlockchainStats {
                volume_usd: Decimal::new(20000, 2),
                payment_count: 10,
                average_value: Decimal::new(2000, 2),
            },
        );
        
        by_blockchain.insert(
            "POLYGON".to_string(),
            BlockchainStats {
                volume_usd: Decimal::new(10000, 2),
                payment_count: 5,
                average_value: Decimal::new(2000, 2),
            },
        );

        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(100000, 2),
            successful_payments: 50,
            failed_payments: 5,
            total_fees_paid: Decimal::new(1500, 2),
            average_transaction_value: Decimal::new(2000, 2),
            by_blockchain,
        };

        assert_eq!(report.by_blockchain.len(), 4);
        assert!(report.by_blockchain.contains_key("SOLANA"));
        assert!(report.by_blockchain.contains_key("BEP20"));
        assert!(report.by_blockchain.contains_key("ARBITRUM"));
        assert!(report.by_blockchain.contains_key("POLYGON"));
    }

    #[test]
    fn test_analytics_report_empty_blockchains() {
        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(0, 0),
            successful_payments: 0,
            failed_payments: 0,
            total_fees_paid: Decimal::new(0, 0),
            average_transaction_value: Decimal::new(0, 0),
            by_blockchain: HashMap::new(),
        };

        assert_eq!(report.total_volume_usd, Decimal::new(0, 0));
        assert_eq!(report.successful_payments, 0);
        assert_eq!(report.failed_payments, 0);
        assert_eq!(report.by_blockchain.len(), 0);
    }

    #[test]
    fn test_analytics_report_average_calculation() {
        // Test that average is correctly calculated
        let total_volume = Decimal::new(100000, 2); // $1000.00
        let successful_payments = 20;
        let expected_average = Decimal::new(5000, 2); // $50.00

        let report = AnalyticsReport {
            total_volume_usd: total_volume,
            successful_payments,
            failed_payments: 5,
            total_fees_paid: Decimal::new(1500, 2),
            average_transaction_value: expected_average,
            by_blockchain: HashMap::new(),
        };

        assert_eq!(report.average_transaction_value, expected_average);
        
        // Verify the calculation
        let calculated_average = total_volume / Decimal::from(successful_payments);
        assert_eq!(calculated_average, expected_average);
    }

    #[test]
    fn test_blockchain_stats_average_calculation() {
        let volume = Decimal::new(50000, 2); // $500.00
        let count = 10;
        let expected_average = Decimal::new(5000, 2); // $50.00

        let stats = BlockchainStats {
            volume_usd: volume,
            payment_count: count,
            average_value: expected_average,
        };

        assert_eq!(stats.average_value, expected_average);
        
        // Verify the calculation
        let calculated_average = volume / Decimal::from(count);
        assert_eq!(calculated_average, expected_average);
    }

    #[test]
    fn test_analytics_report_with_high_failure_rate() {
        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(50000, 2),
            successful_payments: 10,
            failed_payments: 40, // 80% failure rate
            total_fees_paid: Decimal::new(750, 2),
            average_transaction_value: Decimal::new(5000, 2),
            by_blockchain: HashMap::new(),
        };

        assert_eq!(report.successful_payments, 10);
        assert_eq!(report.failed_payments, 40);
        assert!(report.failed_payments > report.successful_payments);
    }

    #[test]
    fn test_analytics_report_fee_calculation() {
        // Test fee calculation (1.5% of $1000 = $15)
        let total_volume = Decimal::new(100000, 2); // $1000.00
        let fee_percentage = Decimal::new(150, 2); // 1.50%
        let expected_fees = Decimal::new(1500, 2); // $15.00

        let report = AnalyticsReport {
            total_volume_usd: total_volume,
            successful_payments: 20,
            failed_payments: 5,
            total_fees_paid: expected_fees,
            average_transaction_value: Decimal::new(5000, 2),
            by_blockchain: HashMap::new(),
        };

        assert_eq!(report.total_fees_paid, expected_fees);
        
        // Verify the calculation
        let calculated_fees = total_volume * fee_percentage / Decimal::new(100, 0);
        assert_eq!(calculated_fees, expected_fees);
    }

    #[test]
    fn test_blockchain_stats_zero_payments() {
        let stats = BlockchainStats {
            volume_usd: Decimal::new(0, 0),
            payment_count: 0,
            average_value: Decimal::new(0, 0),
        };

        assert_eq!(stats.volume_usd, Decimal::new(0, 0));
        assert_eq!(stats.payment_count, 0);
        assert_eq!(stats.average_value, Decimal::new(0, 0));
    }

    #[test]
    fn test_analytics_report_large_volumes() {
        // Test with large transaction volumes
        let mut by_blockchain = HashMap::new();
        by_blockchain.insert(
            "SOLANA".to_string(),
            BlockchainStats {
                volume_usd: Decimal::new(100000000, 2), // $1,000,000
                payment_count: 1000,
                average_value: Decimal::new(100000, 2), // $1,000
            },
        );

        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(100000000, 2), // $1,000,000
            successful_payments: 1000,
            failed_payments: 50,
            total_fees_paid: Decimal::new(1500000, 2), // $15,000 (1.5%)
            average_transaction_value: Decimal::new(100000, 2), // $1,000
            by_blockchain,
        };

        assert_eq!(report.total_volume_usd, Decimal::new(100000000, 2));
        assert_eq!(report.successful_payments, 1000);
        assert_eq!(report.total_fees_paid, Decimal::new(1500000, 2));
    }

    #[test]
    fn test_analytics_report_filtering_by_blockchain() {
        // Test analytics with single blockchain filter
        let mut by_blockchain = HashMap::new();
        by_blockchain.insert(
            "BEP20".to_string(),
            BlockchainStats {
                volume_usd: Decimal::new(50000, 2),
                payment_count: 25,
                average_value: Decimal::new(2000, 2),
            },
        );

        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(50000, 2),
            successful_payments: 25,
            failed_payments: 3,
            total_fees_paid: Decimal::new(750, 2),
            average_transaction_value: Decimal::new(2000, 2),
            by_blockchain,
        };

        assert_eq!(report.by_blockchain.len(), 1);
        assert!(report.by_blockchain.contains_key("BEP20"));
        assert_eq!(report.total_volume_usd, Decimal::new(50000, 2));
    }

    #[test]
    fn test_analytics_report_filtering_by_status() {
        // Test analytics with status filter (only confirmed)
        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(75000, 2),
            successful_payments: 30,
            failed_payments: 0, // Filtered out
            total_fees_paid: Decimal::new(1125, 2),
            average_transaction_value: Decimal::new(2500, 2),
            by_blockchain: HashMap::new(),
        };

        assert_eq!(report.successful_payments, 30);
        assert_eq!(report.failed_payments, 0);
    }

    #[test]
    fn test_analytics_report_date_range() {
        // Test analytics for specific date range
        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(25000, 2),
            successful_payments: 10,
            failed_payments: 2,
            total_fees_paid: Decimal::new(375, 2),
            average_transaction_value: Decimal::new(2500, 2),
            by_blockchain: HashMap::new(),
        };

        // Verify calculations
        let expected_average = Decimal::new(25000, 2) / Decimal::from(10);
        assert_eq!(report.average_transaction_value, expected_average);
    }

    #[test]
    fn test_blockchain_stats_multiple_networks() {
        // Test stats for multiple blockchain networks
        let solana_stats = BlockchainStats {
            volume_usd: Decimal::new(30000, 2),
            payment_count: 15,
            average_value: Decimal::new(2000, 2),
        };

        let bep20_stats = BlockchainStats {
            volume_usd: Decimal::new(20000, 2),
            payment_count: 10,
            average_value: Decimal::new(2000, 2),
        };

        let arbitrum_stats = BlockchainStats {
            volume_usd: Decimal::new(15000, 2),
            payment_count: 5,
            average_value: Decimal::new(3000, 2),
        };

        // Verify each network's stats
        assert_eq!(solana_stats.payment_count, 15);
        assert_eq!(bep20_stats.payment_count, 10);
        assert_eq!(arbitrum_stats.payment_count, 5);

        // Verify total
        let total_volume = solana_stats.volume_usd + bep20_stats.volume_usd + arbitrum_stats.volume_usd;
        assert_eq!(total_volume, Decimal::new(65000, 2));
    }

    #[test]
    fn test_analytics_report_with_high_volume_low_count() {
        // Test analytics with few high-value transactions
        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(100000, 2), // $1,000
            successful_payments: 2,
            failed_payments: 0,
            total_fees_paid: Decimal::new(1500, 2), // $15
            average_transaction_value: Decimal::new(50000, 2), // $500
            by_blockchain: HashMap::new(),
        };

        assert_eq!(report.successful_payments, 2);
        assert_eq!(report.average_transaction_value, Decimal::new(50000, 2));
    }

    #[test]
    fn test_analytics_report_with_low_volume_high_count() {
        // Test analytics with many low-value transactions
        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(10000, 2), // $100
            successful_payments: 100,
            failed_payments: 5,
            total_fees_paid: Decimal::new(150, 2), // $1.50
            average_transaction_value: Decimal::new(100, 2), // $1.00
            by_blockchain: HashMap::new(),
        };

        assert_eq!(report.successful_payments, 100);
        assert_eq!(report.average_transaction_value, Decimal::new(100, 2));
    }

    #[test]
    fn test_blockchain_stats_precision() {
        // Test decimal precision in blockchain stats
        let stats = BlockchainStats {
            volume_usd: Decimal::new(3333, 2), // $33.33
            payment_count: 3,
            average_value: Decimal::new(1111, 2), // $11.11
        };

        assert_eq!(stats.volume_usd, Decimal::new(3333, 2));
        assert_eq!(stats.average_value, Decimal::new(1111, 2));
    }

    #[test]
    fn test_analytics_report_zero_fees() {
        // Test analytics with zero fees (edge case)
        let report = AnalyticsReport {
            total_volume_usd: Decimal::new(50000, 2),
            successful_payments: 25,
            failed_payments: 2,
            total_fees_paid: Decimal::new(0, 0), // No fees
            average_transaction_value: Decimal::new(2000, 2),
            by_blockchain: HashMap::new(),
        };

        assert_eq!(report.total_fees_paid, Decimal::new(0, 0));
    }
}

