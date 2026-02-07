use crate::fixtures::test_app::TestApp;
use serde_json::Value;

#[tokio::test]
async fn create_transcription_for_conference() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("trans1").await;

    // Create conference
    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "subject": "Transcription Test" }))
        .send()
        .await
        .unwrap();
    let conf: Value = resp.json().await.unwrap();
    let conf_id = conf["id"].as_str().unwrap();

    // Create transcription
    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/transcript",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "language": "en-US" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["conference_id"], conf_id);
    assert_eq!(json["language"], "en-US");
    assert_eq!(json["status"], "Processing");
    assert_eq!(json["format"], "Json");
}

#[tokio::test]
async fn list_transcriptions_for_conference() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("trans2").await;

    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "subject": "Trans List" }))
        .send()
        .await
        .unwrap();
    let conf: Value = resp.json().await.unwrap();
    let conf_id = conf["id"].as_str().unwrap();

    // Create 2 transcriptions
    for lang in &["en-US", "de-DE"] {
        app.auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/transcript",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "language": lang }))
        .send()
        .await
        .unwrap();
    }

    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/conference/{}/transcript",
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
async fn get_transcription_detail() {
    let app = TestApp::spawn().await;
    let tenant = app.seed_tenant("trans3").await;

    let resp = app
        .auth_post(
            &format!("/api/tenant/{}/conference", tenant.tenant_id),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "subject": "Trans Detail" }))
        .send()
        .await
        .unwrap();
    let conf: Value = resp.json().await.unwrap();
    let conf_id = conf["id"].as_str().unwrap();

    let resp = app
        .auth_post(
            &format!(
                "/api/tenant/{}/conference/{}/transcript",
                tenant.tenant_id, conf_id
            ),
            &tenant.admin.access_token,
        )
        .json(&serde_json::json!({ "language": "fr-FR" }))
        .send()
        .await
        .unwrap();
    let trans: Value = resp.json().await.unwrap();
    let trans_id = trans["id"].as_str().unwrap();

    // Get detail
    let resp = app
        .auth_get(
            &format!(
                "/api/tenant/{}/conference/{}/transcript/{}",
                tenant.tenant_id, conf_id, trans_id
            ),
            &tenant.admin.access_token,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["language"], "fr-FR");
    assert!(json["segments"].as_array().unwrap().is_empty());
    assert!(json["action_items"].as_array().unwrap().is_empty());
}
