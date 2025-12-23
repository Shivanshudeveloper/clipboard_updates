use crate::db::schemas::payments::{Payment, NewPayment, PaymentStatus};
use sqlx::PgPool;
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;

pub struct PaymentsRepository;

impl PaymentsRepository {
    pub async fn create_payment(pool: &PgPool, new_payment: &NewPayment) -> Result<Payment, sqlx::Error> {
        let now = Utc::now();
        
        println!("💳 Attempting to insert payment into database...");
        
        // #region agent log
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(r"d:\practise\ClipTray\clipboard_updates\.cursor\debug.log") {
            let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"H1","location":"payments_repository.rs:10","message":"create_payment entry","data":{{"firebase_uid":"{}","organization_id":"{}","payment_status":"{}"}},"timestamp":{}}}"#, new_payment.firebase_uid, new_payment.organization_id, new_payment.payment_status.as_str(), chrono::Utc::now().timestamp_millis());
        }
        // #endregion
        
        let result = sqlx::query_as::<_, Payment>(
            r#"
            INSERT INTO payments 
            (stripe_session_id, stripe_payment_intent_id, organization_id, firebase_uid, email, 
             amount_paid, currency, payment_status, plan_type, paid_at, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#
        )
        .bind(&new_payment.stripe_session_id)
        .bind(&new_payment.stripe_payment_intent_id)
        .bind(&new_payment.organization_id)
        .bind(&new_payment.firebase_uid)
        .bind(&new_payment.email)
        .bind(new_payment.amount_paid)
        .bind(&new_payment.currency)
        .bind(&new_payment.payment_status)
        .bind(&new_payment.plan_type)
        .bind(&new_payment.paid_at)
        .bind(new_payment.metadata.as_ref())
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await;
        
        // #region agent log
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(r"d:\practise\ClipTray\clipboard_updates\.cursor\debug.log") {
            match &result {
                Ok(payment) => {
                    let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"H1","location":"payments_repository.rs:30","message":"Payment created SUCCESS","data":{{"payment_id":{},"organization_id":"{}"}},"timestamp":{}}}"#, payment.id, payment.organization_id, chrono::Utc::now().timestamp_millis());
                }
                Err(e) => {
                    let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"H1","location":"payments_repository.rs:30","message":"Payment creation FAILED","data":{{"error":"{}"}},"timestamp":{}}}"#, e, chrono::Utc::now().timestamp_millis());
                }
            }
        }
        // #endregion

        match &result {
            Ok(payment) => println!("✅ Payment inserted successfully - ID: {}, Organization: {}", payment.id, payment.organization_id),
            Err(e) => println!("❌ Payment insertion failed: {}", e),
        }

        result
    }

    pub async fn get_by_firebase_uid(pool: &PgPool, firebase_uid: &str) -> Result<Option<Payment>, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            r#"
            SELECT * FROM payments 
            WHERE firebase_uid = $1 
            ORDER BY created_at DESC 
            LIMIT 1
            "#
        )
        .bind(firebase_uid)
        .fetch_optional(pool)
        .await
    }

    pub async fn has_active_plan(pool: &PgPool, firebase_uid: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM payments 
                WHERE firebase_uid = $1 
                AND payment_status = 'paid'
            )
            "#
        )
        .bind(firebase_uid)
        .fetch_one(pool)
        .await?;
        
        Ok(result)
    }
}

