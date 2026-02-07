use crate::fixtures::test_app::TestApp;
use serde_json::Value;

#[tokio::test]
async fn create_recording_for_conference() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("rec1").await;

    // Create and start conference
    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "subject": "Recording Test" }))
        .send()
        .await
        .unwrap();
    let conf: Value = resp.json().await.unwrap();
    let conf_id = conf["id"].as_str().unwrap();

    app.auth_post(
        &format!(
            "/api/tenant/{}/conference/{}/start",
            tenant.tenant_id, conf_id
        ),
        &tenant.admin.access_token,
    )
    .send()
    .await
    .unwrap();

    // Create recording
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/recording",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "recording_type": "video" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["conference_id"], conf_id);
    assert_eq!(json["recording_type"], "Video");
    assert_eq!(json["status"], "Processing");
}

#[tokio::test]
async fn list_recordings_for_conference() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("rec2").await;

    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "subject": "Recording List" }))
        .send()
        .await
        .unwrap();
    let conf: Value = resp.json().await.unwrap();
    let conf_id = conf["id"].as_str().unwrap();

    // Create 2 recordings
    for rec_type in &["video", "audio"] {
        app.auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/recording",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "recording_type": rec_type }))
        .send()
        .await
        .unwrap();
    }

    // List recordings
    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/conference/{}/recording",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["total"], 2);
}

#[tokio::test]
async fn delete_recording() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("rec3").await;

    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "subject": "Delete Rec" }))
        .send()
        .await
        .unwrap();
    let conf: Value = resp.json().await.unwrap();
    let conf_id = conf["id"].as_str().unwrap();

    // Create recording
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/recording",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap();
    let rec: Value = resp.json().await.unwrap();
    let rec_id = rec["id"].as_str().unwrap();

    // Delete it
    let resp = app
        .auth_delete(
            &format!(
                "/api/tenant/{}/conference/{}/recording/{}",
                tenant.tenant_id, conf_id, rec_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["deleted"], true);
}
