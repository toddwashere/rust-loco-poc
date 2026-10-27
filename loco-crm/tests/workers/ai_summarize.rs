use loco_crm::{
    app::App,
    models::contacts,
    workers::ai_summarize::{Worker, WorkerArgs},
};
use loco_rs::{bgworker::BackgroundWorker, testing::prelude::*};
use serial_test::serial;

use crate::support::create_crm;

#[tokio::test]
#[serial]
async fn writes_a_summary_with_the_mock_provider() {
    let boot = boot_test::<App>().await.unwrap();
    let crm = create_crm(&boot.app_context).await;
    Worker::perform_later(
        &boot.app_context,
        WorkerArgs {
            contact_id: crm.contact.id,
        },
    )
    .await
    .unwrap();
    let saved = contacts::Model::find_by_id(&boot.app_context.db, crm.contact.id)
        .await
        .unwrap();
    assert!(saved.ai_summary.unwrap().starts_with("[mock] Ada Lovelace"));
}
