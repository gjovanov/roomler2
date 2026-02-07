use bson::{doc, oid::ObjectId, DateTime};
use mongodb::Database;
use roomler2_db::models::{Channel, ChannelMember, ChannelType};

use super::base::{BaseDao, DaoResult, PaginatedResult, PaginationParams};

pub struct ChannelDao {
    pub base: BaseDao<Channel>,
    pub members: BaseDao<ChannelMember>,
}

impl ChannelDao {
    pub fn new(db: &Database) -> Self {
        Self {
            base: BaseDao::new(db, Channel::COLLECTION),
            members: BaseDao::new(db, ChannelMember::COLLECTION),
        }
    }

    pub async fn create(
        &self,
        tenant_id: ObjectId,
        name: String,
        channel_type: ChannelType,
        parent_id: Option<ObjectId>,
        creator_id: ObjectId,
        is_private: bool,
    ) -> DaoResult<Channel> {
        let path = if let Some(pid) = parent_id {
            let parent = self.base.find_by_id_in_tenant(tenant_id, pid).await?;
            format!("{}.{}", parent.path, name)
        } else {
            name.clone()
        };

        let now = DateTime::now();
        let channel = Channel {
            id: None,
            tenant_id,
            parent_id,
            channel_type,
            name,
            path,
            topic: None,
            purpose: None,
            icon: None,
            position: 0,
            is_private,
            is_archived: false,
            is_read_only: false,
            is_default: false,
            permission_overwrites: Vec::new(),
            tags: Vec::new(),
            media_settings: None,
            creator_id,
            last_message_id: None,
            last_activity_at: None,
            member_count: 1,
            message_count: 0,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let channel_id = self.base.insert_one(&channel).await?;

        // Auto-join creator
        self.join(tenant_id, channel_id, creator_id).await?;

        self.base.find_by_id(channel_id).await
    }

    pub async fn find_by_tenant(
        &self,
        tenant_id: ObjectId,
    ) -> DaoResult<Vec<Channel>> {
        self.base
            .find_many(
                doc! { "tenant_id": tenant_id, "deleted_at": null },
                Some(doc! { "parent_id": 1, "position": 1 }),
            )
            .await
    }

    pub async fn find_user_channels(
        &self,
        tenant_id: ObjectId,
        user_id: ObjectId,
    ) -> DaoResult<Vec<Channel>> {
        let memberships = self
            .members
            .find_many(
                doc! { "tenant_id": tenant_id, "user_id": user_id },
                None,
            )
            .await?;

        let channel_ids: Vec<ObjectId> = memberships.iter().map(|m| m.channel_id).collect();

        if channel_ids.is_empty() {
            return Ok(Vec::new());
        }

        self.base
            .find_many(
                doc! { "_id": { "$in": channel_ids }, "deleted_at": null },
                Some(doc! { "parent_id": 1, "position": 1 }),
            )
            .await
    }

    pub async fn join(
        &self,
        tenant_id: ObjectId,
        channel_id: ObjectId,
        user_id: ObjectId,
    ) -> DaoResult<ChannelMember> {
        let now = DateTime::now();
        let member = ChannelMember {
            id: None,
            tenant_id,
            channel_id,
            user_id,
            joined_at: now,
            last_read_message_id: None,
            last_read_at: None,
            unread_count: 0,
            mention_count: 0,
            notification_override: None,
            is_muted: false,
            is_pinned: false,
            created_at: now,
            updated_at: now,
        };

        let id = self.members.insert_one(&member).await?;

        // Increment member_count
        self.base
            .update_by_id(
                channel_id,
                doc! { "$inc": { "member_count": 1 } },
            )
            .await?;

        self.members.find_by_id(id).await
    }

    pub async fn leave(
        &self,
        tenant_id: ObjectId,
        channel_id: ObjectId,
        user_id: ObjectId,
    ) -> DaoResult<bool> {
        let deleted = self
            .members
            .hard_delete(doc! {
                "tenant_id": tenant_id,
                "channel_id": channel_id,
                "user_id": user_id,
            })
            .await?;

        if deleted > 0 {
            self.base
                .update_by_id(
                    channel_id,
                    doc! { "$inc": { "member_count": -1 } },
                )
                .await?;
        }

        Ok(deleted > 0)
    }

    pub async fn list_members(
        &self,
        channel_id: ObjectId,
        params: &PaginationParams,
    ) -> DaoResult<PaginatedResult<ChannelMember>> {
        self.members
            .find_paginated(
                doc! { "channel_id": channel_id },
                Some(doc! { "joined_at": 1 }),
                params,
            )
            .await
    }

    pub async fn find_member_user_ids(
        &self,
        channel_id: ObjectId,
    ) -> DaoResult<Vec<ObjectId>> {
        let members = self
            .members
            .find_many(doc! { "channel_id": channel_id }, None)
            .await?;
        Ok(members.into_iter().map(|m| m.user_id).collect())
    }

    pub async fn update(
        &self,
        tenant_id: ObjectId,
        channel_id: ObjectId,
        name: Option<String>,
        topic: Option<String>,
        purpose: Option<String>,
        is_private: Option<bool>,
        is_archived: Option<bool>,
        is_read_only: Option<bool>,
    ) -> DaoResult<bool> {
        let mut set_doc = doc! {};

        if let Some(name) = name {
            set_doc.insert("name", name);
        }
        if let Some(topic) = topic {
            set_doc.insert("topic", doc! { "value": &topic });
        }
        if let Some(purpose) = purpose {
            set_doc.insert("purpose", purpose);
        }
        if let Some(is_private) = is_private {
            set_doc.insert("is_private", is_private);
        }
        if let Some(is_archived) = is_archived {
            set_doc.insert("is_archived", is_archived);
        }
        if let Some(is_read_only) = is_read_only {
            set_doc.insert("is_read_only", is_read_only);
        }

        if set_doc.is_empty() {
            return Ok(false);
        }

        self.base
            .update_one(
                doc! { "_id": channel_id, "tenant_id": tenant_id },
                doc! { "$set": set_doc },
            )
            .await
    }

    pub async fn soft_delete(
        &self,
        tenant_id: ObjectId,
        channel_id: ObjectId,
    ) -> DaoResult<bool> {
        self.base.soft_delete_in_tenant(tenant_id, channel_id).await
    }

    pub async fn explore(
        &self,
        tenant_id: ObjectId,
        query: &str,
    ) -> DaoResult<Vec<Channel>> {
        // Escape regex special chars for safe MongoDB $regex usage
        let escaped: String = query
            .chars()
            .flat_map(|c| {
                if ".*+?^${}()|[]\\".contains(c) {
                    vec!['\\', c]
                } else {
                    vec![c]
                }
            })
            .collect();

        self.base
            .find_many(
                doc! {
                    "tenant_id": tenant_id,
                    "deleted_at": null,
                    "is_private": false,
                    "$or": [
                        { "name": { "$regex": &escaped, "$options": "i" } },
                        { "purpose": { "$regex": &escaped, "$options": "i" } },
                        { "tags": { "$regex": &escaped, "$options": "i" } },
                    ]
                },
                Some(doc! { "member_count": -1 }),
            )
            .await
    }
}
