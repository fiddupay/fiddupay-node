// Trust Score Service
// Calculates merchant trust scores (0-100) based on verified signals

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrustScore {
    pub score: i32,
    pub level: TrustLevel,
    pub breakdown: ScoreBreakdown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TrustLevel {
    Unverified, // 0-59
    Qualified,  // 60-79 (ID Verified)
    Trusted,    // 80-94 (Socials + History)
    Elite,      // 95-100 (Full Business + Long History)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreBreakdown {
    pub identity_verified: bool,
    pub business_verified: bool,
    pub social_score: i32,
    pub history_score: i32,
}

pub struct TrustScoreService;

impl TrustScoreService {
    /// Calculate current trust score for a merchant
    pub fn calculate_score(
        kyc_tier: i32,
        social_handles: &serde_json::Value,
        // In the future: successful_tx_count, account_age_days
    ) -> TrustScore {
        let mut score = 0;
        let mut identity_verified = false;
        let mut business_verified = false;
        let mut social_score = 0;

        // 1. Base Score from KYC Tier (SmileID Pattern)
        // Tier 1 (ID Verified) grants a baseline of 60 points
        if kyc_tier >= 1 {
            score += 60;
            identity_verified = true;
        }

        // Tier 2 (Business Verified) grants an additional 10 points
        if kyc_tier >= 2 {
            score += 10;
            business_verified = true;
        }

        // 2. Social Signals (+10 each, max 30)
        if let Some(handles) = social_handles.as_object() {
            let platforms = ["twitter", "instagram", "linkedin", "facebook", "website"];
            for platform in platforms {
                if let Some(val) = handles.get(platform) {
                    if let Some(s) = val.as_str() {
                        if !s.is_empty() {
                            social_score += 7; // Adjusted to allow 5+ socials to reach 30ish
                        }
                    }
                }
            }
        }

        score += social_score.min(25); // Cap social at 25 points

        // 3. History (Placeholder for now, +5 base for being active)
        let history_score = 5;
        score += history_score;

        let final_score = score.min(100);

        let level = match final_score {
            0..=59 => TrustLevel::Unverified,
            60..=79 => TrustLevel::Qualified,
            80..=94 => TrustLevel::Trusted,
            _ => TrustLevel::Elite,
        };

        TrustScore {
            score: final_score,
            level,
            breakdown: ScoreBreakdown {
                identity_verified,
                business_verified,
                social_score,
                history_score,
            },
        }
    }
}
