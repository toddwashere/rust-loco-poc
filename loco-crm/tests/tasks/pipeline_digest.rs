use loco_crm::app::App;
use loco_rs::{boot::run_task, task, testing::prelude::*};
use serial_test::serial;

use crate::support::{create_crm, mail_count};

#[tokio::test]
#[serial]
async fn emails_the_digest() {
    let boot = boot_test::<App>().await.unwrap();
    create_crm(&boot.app_context).await;
    let before = mail_count(&boot.app_context);
    run_task::<App>(&boot.app_context, Some(&"pipeline_digest".to_string()), &task::Vars::default())
        .await
        .unwrap();
    assert_eq!(mail_count(&boot.app_context), before + 1);
    let mail = boot.app_context.mailer.as_ref().unwrap().deliveries().messages.last().cloned().unwrap();
    assert!(mail.contains("Subject: Pipeline digest: 1 open deals worth $12,500"), "{mail}");
}
