use crate::error::ServiceError;
use crate::models::p2p::*;
use chrono::Utc;
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct P2pService {
    db_pool: PgPool,
}

impl P2pService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn register_profile(
        &self,
        req: &CreateProfileRequest,
    ) -> Result<P2pProfile, ServiceError> {
        // 1. Hash the Password
        use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
        use rand::rngs::OsRng;
        let argon2 = Argon2::default();
        let password_salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &password_salt)
            .map_err(|_| ServiceError::InternalError("Failed to hash password".to_string()))?
            .to_string();

        // 2. Insert profile
        let profile = sqlx::query_as::<_, P2pProfile>(
            r#"
            INSERT INTO p2p_profiles (
                email, nickname, password_hash, first_name, last_name, 
                gender, phone_number, country, terms_accepted
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, email, nickname, password_hash, kyc_level, is_vendor, is_active, sandbox_mode, 
                      total_trades, completion_rate, thumbs_up_count, thumbs_down_count, created_at, updated_at,
                      first_name, last_name, gender, phone_number, country, terms_accepted
            "#
        )
        .bind(&req.email)
        .bind(&req.nickname)
        .bind(&password_hash)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.gender)
        .bind(&req.phone_number)
        .bind(&req.country)
        .bind(req.terms_accepted)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(profile)
    }

    pub async fn get_profile(&self, user_id: i64) -> Result<P2pProfile, ServiceError> {
        let profile = sqlx::query_as::<_, P2pProfile>(
            "SELECT id, email, nickname, password_hash, kyc_level, is_vendor, is_active, sandbox_mode, total_trades, completion_rate, thumbs_up_count, thumbs_down_count, created_at, updated_at, first_name, last_name, gender, phone_number, country, terms_accepted FROM p2p_profiles WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Profile not found".into()))?;

        Ok(profile)
    }

    pub async fn get_balance(
        &self,
        user_id: i64,
        crypto_type: &str,
        sandbox_mode: bool,
    ) -> Result<P2pBalance, ServiceError> {
        let balance = sqlx::query_as::<_, P2pBalance>(
            r#"SELECT id, user_id, crypto_type, available_balance, locked_balance, total_balance, sandbox_mode, last_updated FROM p2p_balances WHERE user_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"#
        )
        .bind(user_id)
        .bind(crypto_type)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(b) = balance {
            Ok(b)
        } else {
            // Return zero balance if record doesn't exist
            Ok(P2pBalance {
                id: 0,
                user_id,
                crypto_type: crypto_type.to_string(),
                available_balance: rust_decimal::Decimal::ZERO,
                locked_balance: rust_decimal::Decimal::ZERO,
                total_balance: rust_decimal::Decimal::ZERO,
                sandbox_mode,
                last_updated: Utc::now(),
            })
        }
    }

    pub async fn create_ad(
        &self,
        user_id: i64,
        request: CreateAdRequest,
        sandbox_mode: bool,
    ) -> Result<P2pAd, ServiceError> {
        // Validation logic can go here (e.g., check if vendor)

        let ad = sqlx::query_as::<_, P2pAd>(
            "INSERT INTO p2p_ads (user_id, ad_type, crypto_type, fiat_currency, price, total_amount, min_limit, max_limit, payment_time_limit, terms_and_conditions, auto_reply, sandbox_mode)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING id, user_id, ad_type, crypto_type, fiat_currency, price, total_amount, min_limit, max_limit, payment_time_limit, status, terms_and_conditions, auto_reply, sandbox_mode, created_at, updated_at"
        )
        .bind(user_id)
        .bind(&request.ad_type)
        .bind(&request.crypto_type)
        .bind(&request.fiat_currency)
        .bind(request.price)
        .bind(request.total_amount)
        .bind(request.min_limit)
        .bind(request.max_limit)
        .bind(request.payment_time_limit.unwrap_or(15))
        .bind(&request.terms_and_conditions)
        .bind(&request.auto_reply)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        // Link payment methods
        for pm_id in request.payment_method_ids {
            sqlx::query(
                "INSERT INTO p2p_ad_payment_methods (ad_id, payment_method_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(ad.id)
            .bind(pm_id)
            .execute(&self.db_pool)
            .await?;
        }

        Ok(ad)
    }

    pub async fn list_ads(
        &self,
        fiat_currency: &str,
        crypto_type: &str,
        ad_type: &str,
        sandbox_mode: bool,
    ) -> Result<Vec<P2pAd>, ServiceError> {
        let ads = sqlx::query_as::<_, P2pAd>(
            "SELECT id, user_id, ad_type, crypto_type, fiat_currency, price, total_amount, min_limit, max_limit, payment_time_limit, status, terms_and_conditions, auto_reply, sandbox_mode, created_at, updated_at 
             FROM p2p_ads 
             WHERE fiat_currency = $1 AND crypto_type = $2 AND ad_type = $3 AND status = 'ACTIVE' AND sandbox_mode = $4
             ORDER BY price ASC"
         )
         .bind(fiat_currency)
         .bind(crypto_type)
         .bind(ad_type)
         .bind(sandbox_mode)
         .fetch_all(&self.db_pool)
         .await?;

        Ok(ads)
    }

    // Trades
    pub async fn create_trade(
        &self,
        taker_id: i64,
        request: CreateTradeRequest,
        sandbox_mode: bool,
    ) -> Result<P2pTrade, ServiceError> {
        let mut tx = self.db_pool.begin().await?;

        // 1. Fetch Ad
        let ad = sqlx::query_as::<_, P2pAd>(
            "SELECT id, user_id, ad_type, crypto_type, fiat_currency, price, total_amount, min_limit, max_limit, payment_time_limit, status, terms_and_conditions, auto_reply, sandbox_mode, created_at, updated_at FROM p2p_ads WHERE id = $1 AND status = 'ACTIVE' FOR UPDATE"
        )
        .bind(request.ad_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Ad not found or inactive".into()))?;

        if ad.user_id == taker_id {
            return Err(ServiceError::ValidationError(
                "Cannot trade with your own Ad".into(),
            ));
        }

        // 2. Calculate Amounts
        let crypto_amount = if let Some(c) = request.crypto_amount {
            c
        } else if let Some(f) = request.fiat_amount {
            f / ad.price
        } else {
            return Err(ServiceError::ValidationError("Must specify amount".into()));
        };
        let fiat_amount = if let Some(f) = request.fiat_amount {
            f
        } else if let Some(c) = request.crypto_amount {
            c * ad.price
        } else {
            return Err(ServiceError::ValidationError("Must specify amount".into()));
        };

        if fiat_amount < ad.min_limit || fiat_amount > ad.max_limit {
            return Err(ServiceError::ValidationError(
                "Amount out of ad limits".into(),
            ));
        }

        let trade_id = uuid::Uuid::new_v4().to_string(); // Or a shorter generated ID
        let expires_at = Utc::now() + chrono::Duration::minutes(ad.payment_time_limit as i64);

        // Determine Maker and Taker
        let (seller_id, _buyer_id) = if ad.ad_type == "SELL" {
            (ad.user_id, taker_id)
        } else {
            (taker_id, ad.user_id)
        };

        // 3. Create Trade record
        let trade = sqlx::query_as::<_, P2pTrade>(
            "INSERT INTO p2p_trades (trade_id, ad_id, maker_id, taker_id, crypto_amount, fiat_amount, price, status, payment_method_id, expires_at, sandbox_mode)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING id, trade_id, ad_id, maker_id, taker_id, crypto_amount, fiat_amount, price, status, payment_method_id, expires_at, paid_at, completed_at, disputed_at, cancel_reason, sandbox_mode, created_at, updated_at"
        )
        .bind(&trade_id)
        .bind(ad.id)
        .bind(ad.user_id)
        .bind(taker_id)
        .bind(crypto_amount)
        .bind(fiat_amount)
        .bind(ad.price)
        .bind("PENDING_PAYMENT")
        .bind(request.payment_method_id)
        .bind(expires_at)
        .bind(sandbox_mode)
        .fetch_one(&mut *tx)
        .await?;

        // 4. Lock funds via Stored Procedure (with Fee)
        sqlx::query("SELECT p2p_lock_funds_in_escrow_with_fee($1, $2, $3, $4, $5)")
            .bind(seller_id)
            .bind(&ad.crypto_type)
            .bind(crypto_amount)
            .bind(&trade_id)
            .bind(sandbox_mode)
            .execute(&mut *tx)
            .await?;

        // 5. Deduct from Ad
        sqlx::query("UPDATE p2p_ads SET total_amount = total_amount - $1 WHERE id = $2")
            .bind(crypto_amount)
            .bind(ad.id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(trade)
    }

    pub async fn release_trade(&self, user_id: i64, trade_id: &str) -> Result<(), ServiceError> {
        let mut tx = self.db_pool.begin().await?;

        let trade = sqlx::query_as::<_, P2pTrade>(
             "SELECT id, trade_id, ad_id, maker_id, taker_id, crypto_amount, fiat_amount, price, status, payment_method_id, expires_at, paid_at, completed_at, disputed_at, cancel_reason, sandbox_mode, created_at, updated_at FROM p2p_trades WHERE trade_id = $1 FOR UPDATE"
        )
        .bind(trade_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Trade not found".into()))?;

        // Determine who the seller is
        let ad_row = sqlx::query("SELECT ad_type, crypto_type FROM p2p_ads WHERE id = $1")
            .bind(trade.ad_id)
            .fetch_one(&mut *tx)
            .await?;
        let ad_type: String = ad_row.get("ad_type");
        let ad_crypto_type: String = ad_row.get("crypto_type");
        let (seller_id, buyer_id) = if ad_type == "SELL" {
            (trade.maker_id, trade.taker_id)
        } else {
            (trade.taker_id, trade.maker_id)
        };

        if user_id != seller_id {
            return Err(ServiceError::Unauthorized(
                "Only the seller can release the trade".into(),
            ));
        }

        if trade.status != "PAID" && trade.status != "DISPUTED" {
            return Err(ServiceError::ValidationError(
                "Trade is not in a releasable state".into(),
            ));
        }

        // Call stored procedure
        sqlx::query("SELECT p2p_release_funds_from_escrow($1, $2, $3, $4, $5, $6)")
            .bind(seller_id)
            .bind(buyer_id)
            .bind(&ad_crypto_type)
            .bind(trade.crypto_amount)
            .bind(&trade.trade_id)
            .bind(trade.sandbox_mode)
            .execute(&mut *tx)
            .await?;

        // Update trade status
        sqlx::query(
            "UPDATE p2p_trades SET status = 'RELEASED', completed_at = NOW() WHERE id = $1",
        )
        .bind(trade.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn submit_rating(
        &self,
        reviewer_id: i64,
        trade_id_str: &str,
        request: CreateRatingRequest,
    ) -> Result<P2pRating, ServiceError> {
        let mut tx = self.db_pool.begin().await?;

        // 1. Fetch trade to ensure it's completed and reviewer was part of it
        let trade_res: Result<Option<P2pTrade>, sqlx::Error> = sqlx::query_as::<_, P2pTrade>(
             "SELECT id, trade_id, ad_id, maker_id, taker_id, crypto_amount, fiat_amount, price, status, payment_method_id, expires_at, paid_at, completed_at, disputed_at, cancel_reason, sandbox_mode, created_at, updated_at 
              FROM p2p_trades WHERE trade_id = $1"
        )
        .bind(trade_id_str)
        .fetch_optional(&mut *tx)
        .await;

        let trade = trade_res?.ok_or_else(|| ServiceError::NotFound("Trade not found".into()))?;

        if trade.status != "RELEASED" && trade.status != "COMPLETED" {
            return Err(ServiceError::ValidationError(
                "Trade must be completed to rate".into(),
            ));
        }

        let target_id = if reviewer_id == trade.maker_id {
            trade.taker_id
        } else if reviewer_id == trade.taker_id {
            trade.maker_id
        } else {
            return Err(ServiceError::Unauthorized(
                "You are not part of this trade".into(),
            ));
        };

        // 2. Insert Rating (will fail on conflict due to UNIQUE constraint)
        let rating_res: Result<P2pRating, sqlx::Error> = sqlx::query_as::<_, P2pRating>(
            "INSERT INTO p2p_ratings (trade_id, reviewer_id, target_id, rating, comment)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, trade_id, reviewer_id, target_id, rating, comment, created_at",
        )
        .bind(trade.id)
        .bind(reviewer_id)
        .bind(target_id)
        .bind(request.rating.to_uppercase())
        .bind(&request.comment)
        .fetch_one(&mut *tx)
        .await;

        let rating = rating_res.map_err(|e| {
            if let sqlx::Error::Database(db_error) = &e {
                if db_error.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    return ServiceError::ValidationError(
                        "You have already rated this trade".into(),
                    );
                }
            }
            ServiceError::DatabaseError(e.to_string())
        })?;

        // 3. Update target profile counts
        if request.rating.to_uppercase() == "THUMBS_UP" {
            sqlx::query(
                "UPDATE p2p_profiles SET thumbs_up_count = thumbs_up_count + 1 WHERE id = $1",
            )
            .bind(target_id)
            .execute(&mut *tx)
            .await?;
        } else if request.rating.to_uppercase() == "THUMBS_DOWN" {
            sqlx::query(
                "UPDATE p2p_profiles SET thumbs_down_count = thumbs_down_count + 1 WHERE id = $1",
            )
            .bind(target_id)
            .execute(&mut *tx)
            .await?;
        } else {
            return Err(ServiceError::ValidationError(
                "Invalid rating type: must be THUMBS_UP or THUMBS_DOWN".into(),
            ));
        }

        // 4. Update completion rate calculation dynamically based on totals
        sqlx::query(
            "UPDATE p2p_profiles 
             SET completion_rate = ROUND(CAST(thumbs_up_count AS NUMERIC) / NULLIF(thumbs_up_count + thumbs_down_count, 0) * 100, 2)
             WHERE id = $1"
        )
        .bind(target_id)
        .execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(rating)
    }

    pub async fn create_support_ticket(
        &self,
        user_id: i64,
        request: CreateSupportTicketRequest,
    ) -> Result<P2pSupportTicket, ServiceError> {
        let ticket = sqlx::query_as::<_, P2pSupportTicket>(
            "INSERT INTO p2p_support_tickets (user_id, subject, category, description, status, reported_user_id, trade_id, attachment_url)
             VALUES ($1, $2, $3, $4, 'OPEN', $5, $6, $7)
             RETURNING id, user_id, subject, category, description, status, reported_user_id, trade_id, attachment_url, admin_notes, created_at, updated_at"
        )
        .bind(user_id)
        .bind(&request.subject)
        .bind(&request.category)
        .bind(&request.description)
        .bind(request.reported_user_id)
        .bind(request.trade_id)
        .bind(&request.attachment_url)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(ticket)
    }
}
