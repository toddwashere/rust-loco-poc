use loco_crm::{
    app::App,
    workers::deal_won::{Worker, WorkerArgs},
};
use loco_rs::{bgworker::BackgroundWorker, testing::prelude::*};
use serial_test::serial;

use crate::support::{create_crm, mail_count};

#[tokio::test]
#[serial]
async fn sends_the_won_email() {
    let boot = boot_test::<App>().await.unwrap();
    let crm = create_crm(&boot.app_context).await;
    let before = mail_count(&boot.app_context);
    Worker::perform_later(
        &boot.app_context,
        WorkerArgs {
            deal_id: crm.deal.id,
        },
    )
    .await
    .unwrap();
    assert_eq!(mail_count(&boot.app_context), before + 1);
}
