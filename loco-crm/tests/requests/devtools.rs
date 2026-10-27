use loco_crm::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

use crate::support::session_cookie;

#[tokio::test]
#[serial]
async fn profiler_records_routes_and_browser_timings() {
    request::<App, _, _>(|request, ctx| async move {
        let (k, v) = session_cookie(&request, &ctx).await;
        assert_eq!(request.get("/_profiler.json").await.status_code(), 303, "requires login");

        request.get("/bench/health").await;
        let rum = request
            .post("/_profiler/rum")
            .json(&serde_json::json!({ "path": "/contacts/42", "ttfb": 12.5, "dom_ready": 40.0, "load": 80.0 }))
            .await;
        assert_eq!(rum.status_code(), 200);

        let snap = request.get("/_profiler.json").add_header(k.clone(), v.clone()).await.json::<serde_json::Value>();
        let routes = snap["routes"].as_array().unwrap();
        assert!(routes.iter().any(|r| r["route"] == "GET /bench/health"));
        assert!(snap["rum"].as_array().unwrap().iter().any(|r| r["path"] == "/contacts/{id}"));

        let page = request.get("/_profiler").add_header(k, v).await;
        assert!(page.text().contains("GET /bench/health"));
    })
    .await;
}
