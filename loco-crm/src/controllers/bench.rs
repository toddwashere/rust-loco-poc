//! Public, unauthenticated endpoints that mirror the Next.js twin exactly, so
//! `bench/run.sh` compares the frameworks, not the features.
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::contacts;

#[derive(Deserialize)]
struct PageQuery {
    page: Option<u64>,
}

#[derive(Serialize)]
struct BenchRow {
    id: i64,
    first_name: String,
    last_name: String,
    email: String,
    company: Option<String>,
}

async fn rows(ctx: &AppContext, page: Option<u64>) -> Result<Vec<BenchRow>> {
    Ok(contacts::Model::page_with_company(&ctx.db, page.unwrap_or(1).max(1), 20)
        .await?
        .into_iter()
        .map(|r| BenchRow {
            id: r.contact.id,
            first_name: r.contact.first_name,
            last_name: r.contact.last_name,
            email: r.contact.email,
            company: r.company.map(|c| c.name),
        })
        .collect())
}

#[debug_handler]
async fn health() -> Result<Response> {
    format::json(json!({ "ok": true }))
}

#[debug_handler]
async fn contacts_json(State(ctx): State<AppContext>, Query(q): Query<PageQuery>) -> Result<Response> {
    format::json(rows(&ctx, q.page).await?)
}

#[debug_handler]
async fn contacts_html(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Query(q): Query<PageQuery>,
) -> Result<Response> {
    let page = q.page.unwrap_or(1).max(1);
    format::render().view(
        &v,
        "bench/contacts.html",
        json!({ "rows": rows(&ctx, Some(page)).await?, "next": page + 1 }),
    )
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/bench")
        .add("/health", get(health))
        .add("/contacts.json", get(contacts_json))
        .add("/contacts", get(contacts_html))
}
