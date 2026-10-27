use loco_crm::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn root_goes_to_dashboard() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/").await;
        assert_eq!(res.status_code(), 303);
        assert_eq!(res.header("location"), "/dashboard");
    })
    .await;
}
