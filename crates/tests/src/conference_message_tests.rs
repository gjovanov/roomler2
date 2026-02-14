use crate::fixtures::test_app::TestApp;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

/// Helper: create + start a conference, return conf_id.
async fn create_and_start_conference(
    app: &TestApp,
    tenant_id: &str,
    token: &str,
    subject: &str,
) -> String {
    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/conference", tenant_id),
            token,
        )
        .json(&serde_json::json!({ "subject": subject }))
        .send()
        .await
        .unwrap();
    let conf: Value = resp.json().await.unwrap();
    let conf_id = conf["id"].as_str().unwrap().to_string();

    app.auth_post(
        &format!("/api/tenant/{}/conference/{}/start", tenant_id, conf_id),
        token,
    )
    .send()
    .await
    .unwrap();

    conf_id
}

#[tokio::test]
async fn create_and_list_conference_chat_messages() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("confmsg1").await;

    let conf_id = create_and_start_conference(
        &app,
        &tenant.tenant_id,
        &tenant.admin.access_token,
        "Chat Test",
    )
    .await;

    // Admin joins conference
    app.auth_post(
        &format!(
            "/api/tenant/{}/conference/{}/join",
            tenant.tenant_id, conf_id
        ),
        &tenant.admin.access_token,
    )
    .send()
    .await
    .unwrap();

    // Send 3 chat messages
    for i in 1..=3 {
        let resp = app
            .auth_post(
                &format!(
                    "/api/tenant/{}/conference/{}/message",
                    tenant.tenant_id, conf_id
                ),
                &tenant.admin.access_token,
            )
            .json(&serde_json::json!({
                "content": format!("Chat message {}", i),
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status().as_u16(), 200, "Failed to create message {}", i);
        let json: Value = resp.json().await.unwrap();
        assert_eq!(json["content"], format!("Chat message {}", i));
        assert!(json["id"].is_string());
        assert!(json["display_name"].is_string());
        assert!(json["created_at"].is_string());
        assert_eq!(json["conference_id"], conf_id);
    }

    // List messages
    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/conference/{}/message",
                tenant.tenant_id, conf_id
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
    // Messages should be sorted by created_at ascending
    assert_eq!(items[0]["content"], "Chat message 1");
    assert_eq!(items[2]["content"], "Chat message 3");
}

#[tokio::test]
async fn non_participant_cannot_send_message() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("confmsg2").await;

    let conf_id = create_and_start_conference(
        &app,
        &tenant.tenant_id,
        &tenant.admin.access_token,
        "Auth Test",
    )
    .await;

    // Admin joins but member does NOT join the conference
    app.auth_post(
        &format!(
            "/api/tenant/{}/conference/{}/join",
            tenant.tenant_id, conf_id
        ),
        &tenant.admin.access_token,
    )
    .send()
    .await
    .unwrap();

    // Member (not a conference participant) tries to send a message
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/message",
                tenant.tenant_id, conf_id
            ),
            &tenant.member.access_token,
        )
        .json(&serde_json::json!({
            "content": "I shouldn't be able to chat",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 403);
}

#[tokio::test]
async fn conference_chat_message_ws_broadcast() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("confmsg3").await;

    let conf_id = create_and_start_conference(
        &app,
        &tenant.tenant_id,
        &tenant.admin.access_token,
        "WS Chat Test",
    )
    .await;

    // Both users join the conference
    app.auth_post(
        &format!(
            "/api/tenant/{}/conference/{}/join",
            tenant.tenant_id, conf_id
        ),
        &tenant.admin.access_token,
    )
    .send()
    .await
    .unwrap();

    app.auth_post(
        &format!(
            "/api/tenant/{}/conference/{}/join",
            tenant.tenant_id, conf_id
        ),
        &tenant.member.access_token,
    )
    .send()
    .await
    .unwrap();

    // Member connects WS
    let ws_url = format!(
        "ws://{}/ws?token={}",
        app.addr, tenant.member.access_token
    );
    let (mut ws_member, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("WS connect failed");

    // Read "connected" message
    ws_member.next().await;

    // Admin sends a chat message via REST
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/message",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "content": "Hello from admin!",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    // Member should receive the WS broadcast
    let msg = tokio::time::timeout(std::time::Duration::from_secs(5), ws_member.next())
        .await
        .expect("Timeout waiting for WS message")
        .unwrap()
        .unwrap();

    let parsed: Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
    assert_eq!(parsed["type"], "conference:message:create");
    assert_eq!(parsed["data"]["content"], "Hello from admin!");
    assert_eq!(parsed["data"]["conference_id"], conf_id);
    assert!(parsed["data"]["display_name"].is_string());

    // Admin connects WS to verify sender exclusion
    let ws_url_admin = format!(
        "ws://{}/ws?token={}",
        app.addr, tenant.admin.access_token
    );
    let (mut ws_admin, _) = tokio_tungstenite::connect_async(&ws_url_admin)
        .await
        .expect("WS connect failed");
    ws_admin.next().await; // connected

    // Admin sends another message
    app.auth_post(
        &format!(
            "/api/tenant/{}/conference/{}/message",
            tenant.tenant_id, conf_id
        ),
        &tenant.admin.access_token,
    )
    .json(&serde_json::json!({
        "content": "Second message",
    }))
    .send()
    .await
    .unwrap();

    // Admin should NOT receive their own message via WS
    let admin_msg =
        tokio::time::timeout(std::time::Duration::from_millis(500), ws_admin.next()).await;
    match admin_msg {
        Ok(Some(Ok(msg))) => {
            let parsed: Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
            assert_ne!(
                parsed["type"], "conference:message:create",
                "Sender should not receive their own chat message via WS"
            );
        }
        _ => {
            // Timeout or closed — correct, admin should not receive their own message
        }
    }

    ws_member.close(None).await.ok();
    ws_admin.close(None).await.ok();
}

#[tokio::test]
async fn cannot_chat_in_ended_conference() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("confmsg4").await;

    let conf_id = create_and_start_conference(
        &app,
        &tenant.tenant_id,
        &tenant.admin.access_token,
        "Ended Chat Test",
    )
    .await;

    // Admin joins
    app.auth_post(
        &format!(
            "/api/tenant/{}/conference/{}/join",
            tenant.tenant_id, conf_id
        ),
        &tenant.admin.access_token,
    )
    .send()
    .await
    .unwrap();

    // End the conference
    app.auth_post(
        &format!(
            "/api/tenant/{}/conference/{}/end",
            tenant.tenant_id, conf_id
        ),
        &tenant.admin.access_token,
    )
    .send()
    .await
    .unwrap();

    // Try to send a chat message — should fail
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/message",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "content": "This should fail",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 400);
}
