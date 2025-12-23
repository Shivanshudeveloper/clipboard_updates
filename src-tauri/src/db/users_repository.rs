use crate::db::schemas::users::{User, NewUser, UpdateUser, UserResponse, PurgeCadence, Plan};
use sqlx::{PgPool};
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;

pub struct UsersRepository;

impl UsersRepository {
    pub async fn create_user(pool: &PgPool, new_user: &NewUser) -> Result<User, sqlx::Error> {
        let now = Utc::now();
        
        println!("🗄️ Attempting to insert user into database...");
        
        // #region agent log
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(r"d:\practise\ClipTray\clipboard_updates\.cursor\debug.log") {
            let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"H1","location":"users_repository.rs:8","message":"create_user entry","data":{{"firebase_uid":"{}","email":"{}"}},"timestamp":{}}}"#, new_user.firebase_uid, new_user.email, chrono::Utc::now().timestamp_millis());
        }
        // #endregion
        
        // #region agent log
        let purge_cadence_bind = PurgeCadence::Never;
        let plan_bind = Plan::Free;
        // let plan_bind_2 = Plan::Free;
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(r"d:\practise\ClipTray\clipboard_updates\.cursor\debug.log") {
            let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"H1","location":"users_repository.rs:26","message":"BEFORE bind values","data":{{"purge_cadence_bind":"{:?}","plan_bind":"{:?}"}},"timestamp":{}}}"#, purge_cadence_bind, plan_bind, chrono::Utc::now().timestamp_millis());
        }
        // #endregion
        
        let result = sqlx::query_as::<_, User>(
            r#"
           INSERT INTO users 
            (firebase_uid, email, display_name, created_at, organization_id, purge_cadence, plan)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(&new_user.firebase_uid)
        .bind(&new_user.email)
        .bind(&new_user.display_name)
        .bind(now)
        .bind(&new_user.organization_id)
        .bind(purge_cadence_bind) // Always default to Never for new users - binds to $6 (purge_cadence)
        .bind(plan_bind) // Binds Plan::Free to $7 (plan column) - CORRECT!
        .fetch_one(pool)
        .await;
        
        // #region agent log
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(r"d:\practise\ClipTray\clipboard_updates\.cursor\debug.log") {
            match &result {
                Ok(_) => {
                    let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"H1","location":"users_repository.rs:30","message":"AFTER query execution - SUCCESS","data":{{}},"timestamp":{}}}"#, chrono::Utc::now().timestamp_millis());
                }
                Err(e) => {
                    let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"H1","location":"users_repository.rs:30","message":"AFTER query execution - ERROR","data":{{"error":"{}"}},"timestamp":{}}}"#, e, chrono::Utc::now().timestamp_millis());
                }
            }
        }
        // #endregion

        match &result {
            Ok(user) => println!("✅ User inserted successfully - ID: {}, Purge Cadence: {}", 
                user.id, user.purge_cadence.to_display_string()),
            Err(e) => println!("❌ Database insertion failed: {}", e),
        }

        result
    }

    /// ✅ Get user by Firebase UID
    pub async fn get_by_firebase_uid(pool: &PgPool, firebase_uid: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE firebase_uid = $1")
            .bind(firebase_uid)
            .fetch_optional(pool)
            .await
    }

    /// ✅ Get all users (with optional limit)
    pub async fn get_all(pool: &PgPool, limit: Option<i64>) -> Result<Vec<User>, sqlx::Error> {
        let limit = limit.unwrap_or(100);
        sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC LIMIT $1")
            .bind(limit)
            .fetch_all(pool)
            .await
    }

    pub async fn get_by_organization_id(
        pool: &PgPool,
        organization_id: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT *
            FROM users
            WHERE organization_id = $1
            LIMIT 1
            "#
        )
        .bind(organization_id)
        .fetch_optional(pool)
        .await
    }


    /// ✅ Update user fields
    pub async fn update_user(pool: &PgPool, id: i64, update: &UpdateUser) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET
                display_name = COALESCE($1, display_name),
                purge_cadence = COALESCE($2, purge_cadence),
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#
        )
        .bind(&update.display_name)
        .bind(&update.purge_cadence)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// ✅ Special method for updating purge settings
    pub async fn update_purge_settings(
        pool: &PgPool, 
        user_id: i64, 
        auto_purge_unpinned: bool,
        purge_cadence: PurgeCadence,
    ) -> Result<User, sqlx::Error> {
        // If auto_purge_unpinned is false, set purge_cadence to Never
        let effective_cadence = if auto_purge_unpinned {
            purge_cadence
        } else {
            PurgeCadence::Never
        };

        sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET purge_cadence = $1, updated_at = NOW() 
            WHERE id = $2 
            RETURNING *
            "#
        )
        .bind(effective_cadence)
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    /// ✅ Update only purge cadence
    pub async fn update_purge_cadence(
        pool: &PgPool, 
        id: i64, 
        purge_cadence: PurgeCadence
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "UPDATE users SET purge_cadence = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
        )
        .bind(purge_cadence)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// ✅ Delete user by ID
    pub async fn delete_user(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// ✅ Get users by purge cadence (useful for batch operations)
    pub async fn get_users_by_purge_cadence(
        pool: &PgPool,
        purge_cadence: PurgeCadence,
    ) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE purge_cadence = $1 AND is_active = true"
        )
        .bind(purge_cadence)
        .fetch_all(pool)
        .await
    }

    /// ✅ Get purge cadence options for frontend
    pub fn get_purge_cadence_options() -> Vec<&'static str> {
        PurgeCadence::all_options()
    }



    /// ✅ Convert user to API response
    pub fn to_response(user: User) -> UserResponse {
        UserResponse::from(user)
    }

    pub async fn update_retain_tags(
        pool: &PgPool,
        user_id: i64,
        retain_tags: bool,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET retain_tags = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#
        )
        .bind(retain_tags)
        .bind(user_id)
        .fetch_one(pool)
        .await
    }
}