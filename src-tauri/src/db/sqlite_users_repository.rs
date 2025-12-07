// src/db/sqlite_users_repository.rs

use crate::db::schemas::users::{User, NewUser, UpdateUser, UserResponse, PurgeCadence};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Error, SqlitePool};
use std::str::FromStr;

    // existing fns...
pub struct SqliteUsersRepository;

/// Internal row mapping for SQLite, stores purge_cadence as TEXT.
#[derive(Debug, Clone, FromRow)]
struct SqliteUserRow {
    pub id: i64,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub organization_id: Option<String>,
    pub purge_cadence: String,
    // Optional extra fields in SQLite schema; they will be ignored by User
    pub updated_at: DateTime<Utc>,
    pub last_login_at: DateTime<Utc>,
    pub retain_tags: bool,
    // If you later add is_active to SQLite, you can include:
    // pub is_active: bool,
}

impl From<SqliteUserRow> for User {
    fn from(row: SqliteUserRow) -> Self {
        let cadence = PurgeCadence::from_str(&row.purge_cadence).unwrap_or_default();
        User {
            id: row.id,
            firebase_uid: row.firebase_uid,
            email: row.email,
            display_name: row.display_name,
            created_at: row.created_at,
            organization_id: row.organization_id,
            purge_cadence: cadence,
            retain_tags: row.retain_tags,
        }
    }
}

impl SqliteUsersRepository {
    pub async fn create_user(pool: &SqlitePool, new_user: &NewUser) -> Result<User, sqlx::Error> {
        let now = Utc::now();

        println!("🗄️ [SQLite] Attempting to insert user into database...");

        let row = sqlx::query_as::<_, SqliteUserRow>(
            r#"
            INSERT INTO users 
                (firebase_uid, email, display_name, created_at, organization_id, purge_cadence)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            RETURNING *
            "#,
        )
        .bind(&new_user.firebase_uid)
        .bind(&new_user.email)
        .bind(&new_user.display_name)
        .bind(now)
        .bind(&new_user.organization_id)
        .bind(PurgeCadence::Never.as_str()) // TEXT in SQLite
        .fetch_one(pool)
        .await;

        match &row {
            Ok(user) => println!(
                "✅ [SQLite] User inserted successfully - ID: {}, Purge Cadence: {}",
                user.id, user.purge_cadence
            ),
            Err(e) => println!("❌ [SQLite] Database insertion failed: {}", e),
        }

        row.map(Into::into)
    }

    /// Get user by Firebase UID
    pub async fn get_by_firebase_uid(
        pool: &SqlitePool,
        firebase_uid: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, SqliteUserRow>(
            "SELECT * FROM users WHERE firebase_uid = ?1",
        )
        .bind(firebase_uid)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Into::into))
    }

    /// Get all users (with optional limit)
    pub async fn get_all(
        pool: &SqlitePool,
        limit: Option<i64>,
    ) -> Result<Vec<User>, sqlx::Error> {
        let limit = limit.unwrap_or(100);
        let rows = sqlx::query_as::<_, SqliteUserRow>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT ?1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }



    pub async fn get_by_organization_id(
        pool: &SqlitePool,
        organization_id: &str,
    ) -> Result<Option<User>, Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, firebase_uid, email, display_name, organization_id,
                   retain_tags, purge_cadence,
                   created_at, updated_at
            FROM users
            WHERE organization_id = ?
            LIMIT 1
            "#
        )
        .bind(organization_id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }



    /// Update user fields (display_name, purge_cadence)
    pub async fn update_user(
        pool: &SqlitePool,
        id: i64,
        update: &UpdateUser,
    ) -> Result<User, sqlx::Error> {
        // Convert Option<PurgeCadence> -> Option<String> for TEXT column
        let purge_cadence_str = update
            .purge_cadence
            .as_ref()
            .map(|c| c.as_str().to_string());

        let row = sqlx::query_as::<_, SqliteUserRow>(
            r#"
            UPDATE users
            SET
                display_name = COALESCE(?1, display_name),
                purge_cadence = COALESCE(?2, purge_cadence),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?3
            RETURNING *
            "#,
        )
        .bind(&update.display_name)
        .bind(&purge_cadence_str)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    /// Special method for updating purge settings
    pub async fn update_purge_settings(
        pool: &SqlitePool,
        user_id: i64,
        auto_purge_unpinned: bool,
        purge_cadence: PurgeCadence,
    ) -> Result<User, sqlx::Error> {
        let effective_cadence = if auto_purge_unpinned {
            purge_cadence
        } else {
            PurgeCadence::Never
        };

        let row = sqlx::query_as::<_, SqliteUserRow>(
            r#"
            UPDATE users 
            SET purge_cadence = ?1, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?2 
            RETURNING *
            "#,
        )
        .bind(effective_cadence.as_str())
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    /// Update only purge cadence
    pub async fn update_purge_cadence(
        pool: &SqlitePool,
        id: i64,
        purge_cadence: PurgeCadence,
    ) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as::<_, SqliteUserRow>(
            r#"
            UPDATE users 
            SET purge_cadence = ?1, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?2 
            RETURNING *
            "#,
        )
        .bind(purge_cadence.as_str())
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    /// Delete user by ID
    pub async fn delete_user(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Get users by purge cadence (offline batch ops)
    pub async fn get_users_by_purge_cadence(
        pool: &SqlitePool,
        purge_cadence: PurgeCadence,
    ) -> Result<Vec<User>, sqlx::Error> {
        // No is_active column in SQLite schema, so no filter here.
        let rows = sqlx::query_as::<_, SqliteUserRow>(
            "SELECT * FROM users WHERE purge_cadence = ?1",
        )
        .bind(purge_cadence.as_str())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Purge cadence options for frontend
    pub fn get_purge_cadence_options() -> Vec<&'static str> {
        PurgeCadence::all_options()
    }

    /// Convert user to API response
    pub fn to_response(user: User) -> UserResponse {
        UserResponse::from(user)
    }

    pub async fn update_retain_tags(
        pool: &SqlitePool,
        user_id: i64,
        retain_tags: bool,
    ) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as::<_, SqliteUserRow>(
            r#"
            UPDATE users
            SET retain_tags = ?1,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?2
            RETURNING *
            "#,
        )
        .bind(retain_tags)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }
}
