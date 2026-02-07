use axum::{Json, extract::{Path, Query, State}};
use bson::{doc, oid::ObjectId};
use serde::Serialize;

use crate::{error::ApiError, extractors::auth::AuthUser, state::AppState};
use roomler2_services::dao::base::PaginationParams;

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub id: String,
    pub user_id: String,
    pub nickname: Option<String>,
    pub role_ids: Vec<String>,
    pub joined_at: String,
}

pub async fn list_members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(tenant_id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    let result = state
        .tenants
        .members
        .find_paginated(
            doc! { "tenant_id": tid },
            Some(doc! { "joined_at": 1 }),
            &params,
        )
        .await?;

    let items: Vec<MemberResponse> = result
        .items
        .into_iter()
        .map(|m| MemberResponse {
            id: m.id.unwrap().to_hex(),
            user_id: m.user_id.to_hex(),
            nickname: m.nickname,
            role_ids: m.role_ids.iter().map(|r| r.to_hex()).collect(),
            joined_at: m.joined_at.try_to_rfc3339_string().unwrap_or_default(),
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
