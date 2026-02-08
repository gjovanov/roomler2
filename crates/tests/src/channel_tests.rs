use crate::fixtures::test_app::TestApp;
use serde_json::Value;

#[tokio::test]
async fn create_channel_and_list() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("chantest").await;

    // List channels (admin sees all 3 seeded channels)
    let resp = app
        .auth_get(
            &format!("/api/tenant/{}/channel", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);

    let channels: Vec<Value> = resp.json().await.unwrap();
    assert_eq!(channels.len(), 3);

    let names: Vec<&str> = channels.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"general"));
    assert!(names.contains(&"engineering"));
    assert!(names.contains(&"random"));
}

#[tokio::test]
async fn create_channel_with_hierarchy() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("hierarchy").await;

    // Create a category
    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/channel", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "name": "development",
            "channel_type": "category",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let category: Value = resp.json().await.unwrap();
    let category_id = category["id"].as_str().unwrap();

    // Create a child channel
    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/channel", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "name": "frontend",
            "channel_type": "text",
            "parent_id": category_id,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let child: Value = resp.json().await.unwrap();
    assert_eq!(child["name"], "frontend");
    assert_eq!(child["parent_id"], category_id);
    assert_eq!(child["path"], "development.frontend");
}

#[tokio::test]
async fn join_and_leave_channel() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("joinleave").await;
    let channel_id = &tenant.channels[0].id;

    // Member joins channel
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/channel/{}/join",
                tenant.tenant_id, channel_id
            ),
            &tenant.member.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["joined"], true);

    // Member leaves channel
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/channel/{}/leave",
                tenant.tenant_id, channel_id
            ),
            &tenant.member.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["left"], true);
}

#[tokio::test]
async fn create_duplicate_channel_returns_409() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("dupch").await;

    // Create a channel
    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/channel", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "name": "unique-channel",
            "channel_type": "text",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);

    // Try to create a channel with the same name — should return 409 Conflict
    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/channel", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "name": "unique-channel",
            "channel_type": "text",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 409);
}

#[tokio::test]
async fn channel_list_returns_plain_array() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("arrfmt").await;

    let resp = app
        .auth_get(
            &format!("/api/tenant/{}/channel", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);

    // Verify the response is a plain JSON array (not wrapped in { items: [...] })
    let body = resp.text().await.unwrap();
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert!(
        parsed.is_array(),
        "Expected plain array, got: {}",
        &body[..100.min(body.len())]
    );
    assert_eq!(parsed.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn explore_channels_returns_plain_array() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("explarr").await;

    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/channel/explore?q=general",
                tenant.tenant_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);

    // Verify the response is a plain JSON array (not wrapped in { items: [...] })
    let body = resp.text().await.unwrap();
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert!(
        parsed.is_array(),
        "Expected plain array, got: {}",
        &body[..100.min(body.len())]
    );
    assert!(parsed.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn member_can_list_channels() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("memlist").await;

    // Member lists channels
    let resp = app
        .auth_get(
            &format!("/api/tenant/{}/channel", tenant.tenant_id),
            &tenant.member.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let channels: Vec<Value> = resp.json().await.unwrap();
    assert_eq!(channels.len(), 3);
}
