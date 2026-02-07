use axum::{Json, extract::{Path, Query, State}};
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, extractors::auth::AuthUser, state::AppState};
use roomler2_db::models::ChannelType;
use roomler2_services::dao::base::PaginationParams;

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(default)]
    pub channel_type: ChannelType,
    pub parent_id: Option<String>,
    #[serde(default)]
    pub is_private: bool,
}

#[derive(Debug, Serialize)]
pub struct ChannelResponse {
    pub id: String,
    pub name: String,
    pub path: String,
    pub channel_type: String,
    pub parent_id: Option<String>,
    pub is_private: bool,
    pub member_count: u32,
    pub message_count: u64,
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(tenant_id): Path<String>,
) -> Result<Json<Vec<ChannelResponse>>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    let channels = state.channels.find_by_tenant(tid).await?;
    let response: Vec<ChannelResponse> = channels.into_iter().map(to_response).collect();

    Ok(Json(response))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(tenant_id): Path<String>,
    Json(body): Json<CreateChannelRequest>,
) -> Result<Json<ChannelResponse>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    let parent_id = body
        .parent_id
        .as_ref()
        .map(|p| ObjectId::parse_str(p))
        .transpose()
        .map_err(|_| ApiError::BadRequest("Invalid parent_id".to_string()))?;

    let channel = state
        .channels
        .create(
            tid,
            body.name,
            body.channel_type,
            parent_id,
            auth.user_id,
            body.is_private,
        )
        .await?;

    Ok(Json(to_response(channel)))
}

pub async fn join(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, channel_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;
    let cid = ObjectId::parse_str(&channel_id)
        .map_err(|_| ApiError::BadRequest("Invalid channel_id".to_string()))?;

    state.channels.join(tid, cid, auth.user_id).await?;

    Ok(Json(serde_json::json!({ "joined": true })))
}

pub async fn leave(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, channel_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;
    let cid = ObjectId::parse_str(&channel_id)
        .map_err(|_| ApiError::BadRequest("Invalid channel_id".to_string()))?;

    state.channels.leave(tid, cid, auth.user_id).await?;

    Ok(Json(serde_json::json!({ "left": true })))
}

pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, channel_id)): Path<(String, String)>,
) -> Result<Json<ChannelResponse>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;
    let cid = ObjectId::parse_str(&channel_id)
        .map_err(|_| ApiError::BadRequest("Invalid channel_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    let channel = state.channels.base.find_by_id_in_tenant(tid, cid).await?;

    Ok(Json(to_response(channel)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub purpose: Option<String>,
    pub is_private: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_read_only: Option<bool>,
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, channel_id)): Path<(String, String)>,
    Json(body): Json<UpdateChannelRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;
    let cid = ObjectId::parse_str(&channel_id)
        .map_err(|_| ApiError::BadRequest("Invalid channel_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    state
        .channels
        .update(
            tid,
            cid,
            body.name,
            body.topic,
            body.purpose,
            body.is_private,
            body.is_archived,
            body.is_read_only,
        )
        .await?;

    Ok(Json(serde_json::json!({ "updated": true })))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, channel_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;
    let cid = ObjectId::parse_str(&channel_id)
        .map_err(|_| ApiError::BadRequest("Invalid channel_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    state.channels.soft_delete(tid, cid).await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, channel_id)): Path<(String, String)>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;
    let cid = ObjectId::parse_str(&channel_id)
        .map_err(|_| ApiError::BadRequest("Invalid channel_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    let result = state.channels.list_members(cid, &params).await?;

    let items: Vec<serde_json::Value> = result
        .items
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id.unwrap().to_hex(),
                "user_id": m.user_id.to_hex(),
                "channel_id": m.channel_id.to_hex(),
                "joined_at": m.joined_at.try_to_rfc3339_string().unwrap_or_default(),
                "unread_count": m.unread_count,
                "is_muted": m.is_muted,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "items": items,
        "total": result.total,
        "page": result.page,
        "per_page": result.per_page,
        "total_pages": result.total_pages,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ExploreQuery {
    pub q: String,
}

pub async fn explore(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(tenant_id): Path<String>,
    Query(query): Query<ExploreQuery>,
) -> Result<Json<Vec<ChannelResponse>>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    let channels = state.channels.explore(tid, &query.q).await?;
    let response: Vec<ChannelResponse> = channels.into_iter().map(to_response).collect();

    Ok(Json(response))
}

fn to_response(c: roomler2_db::models::Channel) -> ChannelResponse {
    ChannelResponse {
        id: c.id.unwrap().to_hex(),
        name: c.name,
        path: c.path,
        channel_type: format!("{:?}", c.channel_type),
        parent_id: c.parent_id.map(|p| p.to_hex()),
        is_private: c.is_private,
        member_count: c.member_count,
        message_count: c.message_count,
    }
}
