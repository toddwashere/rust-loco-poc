use loco_crm::{
    app::App,
    models::contacts::{self, NewContact},
};
use loco_rs::{model::ModelError, testing::prelude::*};
use serial_test::serial;

use crate::support::create_crm;

#[tokio::test]
#[serial]
async fn rejects_invalid_email() {
    let boot = boot_test::<App>().await.unwrap();
    let res = contacts::Model::create_from_form(
        &boot.app_context.db,
        NewContact {
            first_name: "Bob".into(),
            last_name: String::new(),
            email: "nope".into(),
            phone: Some(String::new()),
            title: None,
            company_id: Some(String::new()),
        },
    )
    .await;
    assert!(matches!(res, Err(ModelError::Validation(_))));
}

#[tokio::test]
#[serial]
async fn search_filters_and_joins_company() {
    let boot = boot_test::<App>().await.unwrap();
    create_crm(&boot.app_context).await;
    let db = &boot.app_context.db;

    let (rows, pages) = contacts::Model::search(db, Some("lovelace"), 1, 20).await.unwrap();
    assert_eq!((rows.len(), pages), (1, 1));
    assert_eq!(rows[0].company.as_ref().unwrap().name, "Acme Corp");

    let (none, _) = contacts::Model::search(db, Some("zzz"), 1, 20).await.unwrap();
    assert!(none.is_empty());
}

#[tokio::test]
#[serial]
async fn dossier_includes_deals_and_notes() {
    let boot = boot_test::<App>().await.unwrap();
    let crm = create_crm(&boot.app_context).await;
    let db = &boot.app_context.db;
    loco_crm::models::notes::Model::create_for_contact(db, crm.contact.id, "Met at conference")
        .await
        .unwrap();
    let dossier = crm.contact.dossier(db).await.unwrap();
    assert!(dossier.contains("Acme rollout (proposal, $12500)"));
    assert!(dossier.contains("Met at conference"));
}
