// Trust Score Service
// Calculates merchant trust scores (0-100) based on verified signals

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrustScore {
    pub score: i32,
    pub tier: String,
    pub identity_verified: bool,
    pub social_verified: bool,
    pub business_verified: bool,
}

pub struct TrustScoreService;

impl TrustScoreService {
    /// Calculate current trust score for a merchant
    pub fn calculate_score(kyc_tier: i32, social_handles: &serde_json::Value) -> TrustScore {
        let mut score = 0;
        let mut identity_verified = false;
        let mut business_verified = false;
        let mut social_verified = false;
        let mut social_count = 0;

        // 1. Base Score from KYC Tier
        // Tier 1 (Silver) grants a baseline of 60 points
        if kyc_tier >= 1 {
            score += 60;
            identity_verified = true;
        }

        // Tier 2 (Gold) grants an additional 15 points
        if kyc_tier >= 2 {
            score += 15;
            business_verified = true;
        }

        // 2. Social Signals (+5 each, max 20)
        if let Some(handles) = social_handles.as_object() {
            let platforms = ["twitter", "instagram", "linkedin", "facebook", "website"];
            for platform in platforms {
                if let Some(val) = handles.get(platform) {
                    if let Some(s) = val.as_str() {
                        if !s.is_empty() {
                            social_count += 1;
                        }
                    }
                }
            }
        }

        if social_count >= 1 {
            social_verified = true;
            score += (social_count * 5).min(20);
        }

        // 3. History (Placeholder: +5 for being active)
        score += 5;

        let final_score = score.min(100);

        let tier = match kyc_tier {
            t if t >= 2 => "Gold".to_string(),
            1 => "Silver".to_string(),
            _ => "Sandbox".to_string(),
        };

        TrustScore {
            score: final_score,
            tier,
            identity_verified,
            social_verified,
            business_verified,
        }
    }
}
