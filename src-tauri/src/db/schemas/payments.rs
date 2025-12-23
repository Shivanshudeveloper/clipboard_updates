use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Paid,
    Unpaid,
    Failed,
}

// Custom implementation for VARCHAR(20) in PostgreSQL (not enum type)
impl sqlx::Type<sqlx::Postgres> for PaymentStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for PaymentStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s: &str = sqlx::Decode::<sqlx::Postgres>::decode(value)?;
        match s {
            "paid" => Ok(PaymentStatus::Paid),
            "unpaid" => Ok(PaymentStatus::Unpaid),
            "failed" => Ok(PaymentStatus::Failed),
            _ => Err(format!("Invalid payment_status: {}", s).into()),
        }
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for PaymentStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str(), buf)
    }
}

impl Default for PaymentStatus {
    fn default() -> Self {
        Self::Unpaid
    }
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Paid => "paid",
            Self::Unpaid => "unpaid",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: i64,
    pub stripe_session_id: String,
    pub stripe_payment_intent_id: Option<String>,
    pub organization_id: String,
    pub firebase_uid: String,
    pub email: String,
    pub amount_paid: i32, // Stored as cents (e.g., 2900 for $29.00)
    pub currency: String,
    pub payment_status: PaymentStatus,
    pub plan_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPayment {
    pub stripe_session_id: String,
    pub stripe_payment_intent_id: Option<String>,
    pub organization_id: String,
    pub firebase_uid: String,
    pub email: String,
    pub amount_paid: i64, // Stored as cents
    pub currency: String,
    pub payment_status: PaymentStatus,
    pub plan_type: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

