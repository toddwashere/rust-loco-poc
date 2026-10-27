use loco_crm::{app::App, models::_entities::companies};
use loco_rs::{model::ModelError, testing::prelude::*};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn company_requires_a_name() {
    let boot = boot_test::<App>().await.unwrap();
    let res = companies::ActiveModel {
        name: Set(String::new()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await;
    // before_save validation surfaces as a DbErr from insert.
    assert!(res.is_err());
    let _ = ModelError::EntityNotFound;
}
