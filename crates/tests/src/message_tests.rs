use crate::fixtures::test_app::TestApp;
use serde_json::Value;

#[tokio::test]
async fn create_and_list_messages() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("msgtest").await;
    let channel_id = &tenant.channels[0].id;

    // Admin joins the channel first
    app.auth_post(
        &format!(
            "/api/tenant/{}/channel/{}/join",
            tenant.tenant_id, channel_id
        ),
        &tenant.admin.access_token,
    )
    .send()
    .await
    .unwrap();

    // Create messages
    for i in 1..=3 {
        let resp = app
            .auth_post(
                &format!(
                    "/api/tenant/{}/channel/{}/message",
                    tenant.tenant_id, channel_id
                ),
                &tenant.admin.access_token,
            )
            .json(&serde_json::json!({
                "content": format!("Hello message {}", i),
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status().as_u16(), 200, "Failed to create message {}", i);
    }

    // List messages (paginated response)
    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/channel/{}/message",
                tenant.tenant_id, channel_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["total"], 3);
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
}

#[tokio::test]
async fn update_message() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("msgedit").await;
    let channel_id = &tenant.channels[0].id;

    // Join channel
    app.auth_post(
        &format!(
            "/api/tenant/{}/channel/{}/join",
            tenant.tenant_id, channel_id
        ),
        &tenant.admin.access_token,
    )
    .send()
    .await
    .unwrap();

    // Create a message
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/channel/{}/message",
                tenant.tenant_id, channel_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "content": "Original message",
        }))
        .send()
        .await
        .unwrap();

    let msg: Value = resp.json().await.unwrap();
    let message_id = msg["id"].as_str().unwrap();

    // Update the message
    let resp = app
        .auth_put(
            &format!(
                "/api/tenant/{}/channel/{}/message/{}",
                tenant.tenant_id, channel_id, message_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "content": "Updated message",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["updated"], true);
}

#[tokio::test]
async fn delete_message_soft_deletes() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("msgdel").await;
    let channel_id = &tenant.channels[0].id;

    // Join channel
    app.auth_post(
        &format!(
            "/api/tenant/{}/channel/{}/join",
            tenant.tenant_id, channel_id
        ),
        &tenant.admin.access_token,
    )
    .send()
    .await
    .unwrap();

    // Create a message
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/channel/{}/message",
                tenant.tenant_id, channel_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "content": "To be deleted",
        }))
        .send()
        .await
        .unwrap();

    let msg: Value = resp.json().await.unwrap();
    let message_id = msg["id"].as_str().unwrap();

    // Delete
    let resp = app
        .auth_delete(
            &format!(
                "/api/tenant/{}/channel/{}/message/{}",
                tenant.tenant_id, channel_id, message_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);

    // List messages - should be empty (soft deleted not returned)
    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/channel/{}/message",
                tenant.tenant_id, channel_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["total"], 0);
    assert_eq!(json["items"].as_array().unwrap().len(), 0);
}
