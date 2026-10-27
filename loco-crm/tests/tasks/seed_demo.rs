use loco_crm::{app::App, models::_entities::contacts, models::users, tasks::seed_demo};
use loco_rs::{boot::run_task, task, testing::prelude::*};
use sea_orm::{EntityTrait, PaginatorTrait};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn seeds_demo_user_and_data_once() {
    let boot = boot_test::<App>().await.unwrap();
    let ctx = &boot.app_context;
    let vars = task::Vars::from_cli_args(vec![("contacts".into(), "120".into()), ("companies".into(), "10".into())]);
    run_task::<App>(ctx, Some(&"seed_demo".to_string()), &vars).await.unwrap();
    // Idempotent: a second run does not duplicate.
    run_task::<App>(ctx, Some(&"seed_demo".to_string()), &vars).await.unwrap();

    assert_eq!(contacts::Entity::find().count(&ctx.db).await.unwrap(), 120);
    let user = users::Model::find_by_email(&ctx.db, seed_demo::DEMO_EMAIL).await.unwrap();
    assert!(user.verify_password(seed_demo::DEMO_PASSWORD));
}
