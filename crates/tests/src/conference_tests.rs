use crate::fixtures::test_app::TestApp;
use serde_json::Value;

#[tokio::test]
async fn create_conference() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("conf1").await;

    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "subject": "Team Standup",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["subject"], "Team Standup");
    assert_eq!(json["status"], "Scheduled");
    assert!(json["meeting_code"].as_str().unwrap().len() > 0);
    assert_eq!(json["participant_count"], 0);
}

#[tokio::test]
async fn conference_lifecycle_start_join_leave_end() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("conflife").await;

    // Create conference
    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({
            "subject": "Sprint Planning",
        }))
        .send()
        .await
        .unwrap();

    let conf: Value = resp.json().await.unwrap();
    let conf_id = conf["id"].as_str().unwrap();

    // Start conference
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/start",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["started"], true);

    // Get conference - check status is InProgress
    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/conference/{}",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["status"], "InProgress");

    // Join conference
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/join",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["joined"], true);

    // List participants
    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/conference/{}/participant",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let parts: Vec<Value> = resp.json().await.unwrap();
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0]["user_id"], tenant.admin.id);

    // Leave conference
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/leave",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    // End conference
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/end",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["ended"], true);

    // Get conference - check status is Ended
    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/conference/{}",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["status"], "Ended");
}

#[tokio::test]
async fn list_conferences() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("conflist").await;

    // Create 2 conferences
    for subject in &["Standup", "Retro"] {
        app.auth_post(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "subject": subject }))
        .send()
        .await
        .unwrap();
    }

    let resp = app
        .auth_get(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["total"], 2);
}
