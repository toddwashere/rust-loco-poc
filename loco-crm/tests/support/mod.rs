//! Shared fixtures for the CRM tests.
use axum::http::{HeaderName, HeaderValue};
use loco_crm::models::_entities::{companies, contacts, deals};
use loco_rs::{app::AppContext, TestServer};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};

#[allow(dead_code)]
pub struct Crm {
    pub company: companies::Model,
    pub contact: contacts::Model,
    pub deal: deals::Model,
}

pub async fn create_crm(ctx: &AppContext) -> Crm {
    let company = companies::ActiveModel {
        name: Set("Acme Corp".into()),
        domain: Set(Some("acme.example".into())),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap();
    let contact = contacts::ActiveModel {
        first_name: Set("Ada".into()),
        last_name: Set("Lovelace".into()),
        email: Set("ada@acme.example".into()),
        title: Set(Some("CTO".into())),
        company_id: Set(Some(company.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap();
    let deal = deals::ActiveModel {
        title: Set("Acme rollout".into()),
        amount_cents: Set(Some(1_250_000)),
        stage: Set("proposal".into()),
        contact_id: Set(Some(contact.id)),
        company_id: Set(Some(company.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap();
    Crm {
        company,
        contact,
        deal,
    }
}

/// Register + log in through the JSON API and return the session cookie
/// header the server-rendered pages read.
pub async fn session_cookie(request: &TestServer, ctx: &AppContext) -> (HeaderName, HeaderValue) {
    request
        .post("/api/auth/register")
        .json(&serde_json::json!({ "name": "Tess", "email": "tess@example.com", "password": "pass1234" }))
        .await;
    let res = request
        .post("/api/auth/login")
        .json(&serde_json::json!({ "email": "tess@example.com", "password": "pass1234" }))
        .await;
    let token = res.json::<serde_json::Value>()["token"]
        .as_str()
        .unwrap()
        .to_string();
    let _ = ctx;
    (
        HeaderName::from_static("cookie"),
        HeaderValue::from_str(&format!("auth_token={token}")).unwrap(),
    )
}

pub fn htmx() -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("hx-request"),
        HeaderValue::from_static("true"),
    )
}

pub fn mail_count(ctx: &AppContext) -> usize {
    ctx.mailer.as_ref().map_or(0, |m| m.deliveries().count)
}
