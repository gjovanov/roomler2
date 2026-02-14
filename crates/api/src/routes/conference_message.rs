use axum::{Json, extract::{Path, Query, State}};
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, extractors::auth::AuthUser, state::AppState};
use roomler2_db::models::ConferenceStatus;
use roomler2_services::dao::base::PaginationParams;

#[derive(Debug, Deserialize)]
pub struct CreateConferenceChatMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ConferenceChatMessageResponse {
    pub id: String,
    pub conference_id: String,
    pub author_id: String,
    pub display_name: String,
    pub content: String,
    pub created_at: String,
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, conference_id)): Path<(String, String)>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;
    let confid = ObjectId::parse_str(&conference_id)
        .map_err(|_| ApiError::BadRequest("Invalid conference_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    // Verify user is a conference participant
    let participant_ids = state.conferences.find_participant_user_ids(confid).await?;
    if !participant_ids.contains(&auth.user_id) {
        return Err(ApiError::Forbidden("Not a conference participant".to_string()));
    }

    let result = state.conferences.find_chat_messages(confid, &params).await?;

    let items: Vec<ConferenceChatMessageResponse> = result
        .items
        .into_iter()
        .map(to_response)
        .collect();

    Ok(Json(serde_json::json!({
        "items": items,
        "total": result.total,
        "page": result.page,
        "per_page": result.per_page,
        "total_pages": result.total_pages,
    })))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, conference_id)): Path<(String, String)>,
    Json(body): Json<CreateConferenceChatMessageRequest>,
) -> Result<Json<ConferenceChatMessageResponse>, ApiError> {
    let tid = ObjectId::parse_str(&tenant_id)
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id".to_string()))?;
    let confid = ObjectId::parse_str(&conference_id)
        .map_err(|_| ApiError::BadRequest("Invalid conference_id".to_string()))?;

    if !state.tenants.is_member(tid, auth.user_id).await? {
        return Err(ApiError::Forbidden("Not a member".to_string()));
    }

    // Verify user is a conference participant
    let participant_ids = state.conferences.find_participant_user_ids(confid).await?;
    if !participant_ids.contains(&auth.user_id) {
        return Err(ApiError::Forbidden("Not a conference participant".to_string()));
    }

    // Verify conference is InProgress
    let conference = state.conferences.base.find_by_id_in_tenant(tid, confid).await?;
    if !matches!(conference.status, ConferenceStatus::InProgress) {
        return Err(ApiError::BadRequest(
            "Chat is only available during an active conference".to_string(),
        ));
    }

    // Lookup user display_name
    let user = state.users.base.find_by_id(auth.user_id).await?;

    let msg = state
        .conferences
        .create_chat_message(tid, confid, auth.user_id, user.display_name, body.content)
        .await?;

    let response = to_response(msg);

    // Broadcast to all participants except sender
    let recipients: Vec<ObjectId> = participant_ids
        .into_iter()
        .filter(|id| *id != auth.user_id)
        .collect();
    let event = serde_json::json!({
        "type": "conference:message:create",
        "data": &response,
    });
    crate::ws::dispatcher::broadcast(&state.ws_storage, &recipients, &event).await;

    Ok(Json(response))
}

fn to_response(m: roomler2_db::models::ConferenceChatMessage) -> ConferenceChatMessageResponse {
    ConferenceChatMessageResponse {
        id: m.id.unwrap().to_hex(),
        conference_id: m.conference_id.to_hex(),
        author_id: m.author_id.to_hex(),
        display_name: m.display_name,
        content: m.content,
        created_at: m.created_at.try_to_rfc3339_string().unwrap_or_default(),
    }
}
