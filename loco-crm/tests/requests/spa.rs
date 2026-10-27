use loco_crm::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn spa_shell_is_served_for_client_routes() {
    request::<App, _, _>(|request, _ctx| async move {
        for path in ["/app", "/app/contacts/12/edit"] {
            let res = request.get(path).await;
            assert_eq!(res.status_code(), 200, "{path}");
            let body = res.text();
            assert!(body.contains("id=\"root\"") || body.contains("not built yet"));
        }
    })
    .await;
}
