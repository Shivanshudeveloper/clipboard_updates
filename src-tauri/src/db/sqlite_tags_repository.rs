use sqlx::{Error, SqlitePool, Row, FromRow};
use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};

use crate::db::schemas::tags::{Tag, NewTag, UpdateTag, TagStats};

pub struct SqliteTagRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LocalTag {
    pub id: i64,
    pub organization_id: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sync_status: String,
    pub server_id: Option<i64>,
}


impl SqliteTagRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_organization_tags(
        &self,
        organization_id: &str,
    ) -> Result<Vec<Tag>, Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, name, color, created_at, updated_at
            FROM tags 
            WHERE organization_id = ?1
            ORDER BY name ASC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;

        let mut tags = Vec::new();
        for row in rows {
            let tag = Tag {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                name: row.get("name"),
                color: row.get("color"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            tags.push(tag);
        }
        Ok(tags)
    }

    pub async fn get_organization_tags_with_server_id(
        &self,
        organization_id: &str,
    ) -> Result<Vec<LocalTag>, Error> {
        let tags = sqlx::query_as::<_, LocalTag>(
            r#"
            SELECT 
                id,
                organization_id,
                name,
                color,
                created_at,
                updated_at,
                sync_status,
                server_id
            FROM tags 
            WHERE organization_id = ?1
            ORDER BY name ASC
            "#
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    pub async fn get_tag(
        &self,
        tag_id: i64,
        organization_id: &str,
    ) -> Result<Option<Tag>, Error> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, name, color, created_at, updated_at
            FROM tags 
            WHERE id = ?1 AND organization_id = ?2
            "#,
        )
        .bind(tag_id)
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let tag = Tag {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                name: row.get("name"),
                color: row.get("color"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(tag))
        } else {
            Ok(None)
        }
    }

    pub async fn create_tag(&self, new_tag: &NewTag) -> Result<Tag, Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO tags (organization_id, name, color, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING id, organization_id, name, color, created_at, updated_at
            "#,
        )
        .bind(&new_tag.organization_id)
        .bind(&new_tag.name)
        .bind(&new_tag.color)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(Tag {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            name: row.get("name"),
            color: row.get("color"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn update_tag(
        &self,
        tag_id: i64,
        organization_id: &str,
        updates: &UpdateTag,
    ) -> Result<Option<Tag>, Error> {
        // Get current tag
        let current_tag = self.get_tag(tag_id, organization_id).await?;

        if let Some(mut tag) = current_tag {
            if let Some(name) = &updates.name {
                tag.name = name.clone();
            }
            if let Some(color) = &updates.color {
                tag.color = color.clone();
            }
            tag.updated_at = Utc::now();

            let row = sqlx::query(
                r#"
                UPDATE tags 
                SET name = ?1, color = ?2, updated_at = ?3, sync_status = 'local'
                WHERE id = ?4 AND organization_id = ?5
                RETURNING id, organization_id, name, color, created_at, updated_at
                "#,
            )
            .bind(&tag.name)
            .bind(&tag.color)
            .bind(tag.updated_at)
            .bind(tag_id)
            .bind(organization_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(row) = row {
                let updated_tag = Tag {
                    id: row.get("id"),
                    organization_id: row.get("organization_id"),
                    name: row.get("name"),
                    color: row.get("color"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(updated_tag))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn delete_tag(
        &self,
        tag_id: i64,
        organization_id: &str,
    ) -> Result<bool, Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM tags 
            WHERE id = ?1 AND organization_id = ?2
            "#,
        )
        .bind(tag_id)
        .bind(organization_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn tag_name_exists(
        &self,
        organization_id: &str,
        name: &str,
    ) -> Result<bool, Error> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM tags 
            WHERE organization_id = ?1 AND LOWER(name) = LOWER(?2)
            "#,
        )
        .bind(organization_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    pub async fn get_tag_stats(
        &self,
        organization_id: &str,
    ) -> Result<Vec<TagStats>, Error> {
        let tags = self.get_organization_tags(organization_id).await?;

        let stats = tags
            .into_iter()
            .map(|tag| TagStats {
                tag_id: tag.id,
                tag_name: tag.name,
                usage_count: 0,
                last_used_at: None,
            })
            .collect();

        Ok(stats)
    }

        /// Get tags that were created/updated locally and not yet synced to cloud
    pub async fn get_pending_sync_tags_for_org(
        &self,
        organization_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<LocalTag>, Error> {
        let limit = limit.unwrap_or(500);

        let tags = sqlx::query_as::<_, LocalTag>(
            r#"
            SELECT 
                id,
                organization_id,
                name,
                color,
                created_at,
                updated_at,
                sync_status,
                server_id
            FROM tags
            WHERE organization_id = ?1
              AND sync_status = 'local'
            ORDER BY id ASC
            LIMIT ?2
            "#
        )
        .bind(organization_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    /// Mark local tag row as synced and store its cloud id
    pub async fn mark_as_synced(
        &self,
        local_id: i64,
        server_id: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE tags
            SET sync_status = 'synced',
                server_id = ?1
            WHERE id = ?2
            "#
        )
        .bind(server_id)
        .bind(local_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

}
