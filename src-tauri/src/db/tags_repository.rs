// src/db/tags_repository.rs
use sqlx::{Error, Pool, Postgres, Row};
use crate::db::schemas::tags::{Tag, NewTag, UpdateTag, TagStats};
use chrono::{Utc};

pub struct TagRepository {
    pool: Pool<Postgres>,
}

impl TagRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn get_organization_tags(&self, organization_id: &str) -> Result<Vec<Tag>, Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, name, color, created_at, updated_at
            FROM tags 
            WHERE organization_id = $1 
            ORDER BY name ASC
            "#
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

    pub async fn get_tag(&self, tag_id: i64, organization_id: &str) -> Result<Option<Tag>, Error> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, name, color, created_at, updated_at
            FROM tags 
            WHERE id = $1 AND organization_id = $2
            "#
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
        let row = sqlx::query(
            r#"
            INSERT INTO tags (organization_id, name, color, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, organization_id, name, color, created_at, updated_at
            "#
        )
        .bind(&new_tag.organization_id)
        .bind(&new_tag.name)
        .bind(&new_tag.color)
        .bind(Utc::now())
        .bind(Utc::now())
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

    pub async fn update_tag(&self, tag_id: i64, organization_id: &str, updates: &UpdateTag) -> Result<Option<Tag>, Error> {
        // First get the current tag
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
                SET name = $1, color = $2, updated_at = $3
                WHERE id = $4 AND organization_id = $5
                RETURNING id, organization_id, name, color, created_at, updated_at
                "#
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

    pub async fn delete_tag(&self, tag_id: i64, organization_id: &str) -> Result<bool, Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM tags 
            WHERE id = $1 AND organization_id = $2
            "#
        )
        .bind(tag_id)
        .bind(organization_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn tag_name_exists(&self, organization_id: &str, name: &str) -> Result<bool, Error> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM tags 
            WHERE organization_id = $1 AND LOWER(name) = LOWER($2)
            "#
        )
        .bind(organization_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    pub async fn get_tag_stats(&self, organization_id: &str) -> Result<Vec<TagStats>, Error> {
        let tags = self.get_organization_tags(organization_id).await?;
        
        let stats = tags.into_iter().map(|tag| TagStats {
            tag_id: tag.id,
            tag_name: tag.name,
            usage_count: 0,
            last_used_at: None,
        }).collect();
        
        Ok(stats)
    }
}