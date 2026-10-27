use loco_crm::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

use crate::support::create_crm;

#[tokio::test]
#[serial]
async fn bench_endpoints_match_the_nextjs_twin_shape() {
    request::<App, _, _>(|request, ctx| async move {
        create_crm(&ctx).await;
        let health = request.get("/bench/health").await;
        assert_eq!(health.json::<serde_json::Value>(), serde_json::json!({ "ok": true }));

        let rows = request.get("/bench/contacts.json?page=1").await.json::<serde_json::Value>();
        let first = &rows[0];
        for key in ["id", "first_name", "last_name", "email", "company"] {
            assert!(first.get(key).is_some(), "missing {key}");
        }

        let html = request.get("/bench/contacts").await;
        assert!(html.text().contains("Ada Lovelace") || html.text().contains("Ada"));
    })
    .await;
}
