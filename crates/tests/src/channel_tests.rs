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
