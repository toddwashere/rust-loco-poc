use loco_crm::{app::App, models::notes};
use loco_rs::{model::ModelError, testing::prelude::*};
use serial_test::serial;

use crate::support::create_crm;

#[tokio::test]
#[serial]
async fn blank_notes_are_rejected_and_recent_joins_contact() {
    let boot = boot_test::<App>().await.unwrap();
    let crm = create_crm(&boot.app_context).await;
    let db = &boot.app_context.db;

    let blank = notes::Model::create_for_contact(db, crm.contact.id, "   ").await;
    assert!(matches!(blank, Err(ModelError::Validation(_))));

    notes::Model::create_for_contact(db, crm.contact.id, "Follow up Tuesday").await.unwrap();
    let recent = notes::Model::recent(db, 5).await.unwrap();
    assert_eq!(recent[0].contact.as_ref().unwrap().first_name, "Ada");
}
