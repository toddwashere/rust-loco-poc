use loco_crm::{app::App, models::deals};
use loco_rs::{model::ModelError, testing::prelude::*};
use sea_orm::IntoActiveModel;
use serial_test::serial;

use crate::support::create_crm;

#[tokio::test]
#[serial]
async fn pipeline_summary_aggregates_per_stage_in_board_order() {
    let boot = boot_test::<App>().await.unwrap();
    create_crm(&boot.app_context).await;
    let summary = deals::Model::pipeline_summary(&boot.app_context.db).await.unwrap();
    let stages: Vec<_> = summary.iter().map(|s| s.stage.as_str()).collect();
    assert_eq!(stages, deals::STAGES);
    let proposal = summary.iter().find(|s| s.stage == "proposal").unwrap();
    assert_eq!((proposal.count, proposal.total_cents), (1, 1_250_000));
    assert_eq!(summary.iter().find(|s| s.stage == "won").unwrap().count, 0);
}

#[tokio::test]
#[serial]
async fn unknown_stage_is_rejected() {
    let boot = boot_test::<App>().await.unwrap();
    let crm = create_crm(&boot.app_context).await;
    let res = crm
        .deal
        .into_active_model()
        .move_to_stage(&boot.app_context.db, "maybe")
        .await;
    assert!(matches!(res, Err(ModelError::Validation(_))));
}
