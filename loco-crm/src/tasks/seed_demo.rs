//! `cargo loco task seed_demo [companies:50] [contacts:1000]`
//!
//! Creates a demo login and a deterministic data set. Contact/company naming
//! matches the Next.js twin's seed so benchmark queries return the same rows.
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait};

use crate::models::{
    _entities::{companies, contacts, deals, notes},
    deals::STAGES,
    users::{self, RegisterParams},
};

pub const DEMO_EMAIL: &str = "demo@example.com";
pub const DEMO_PASSWORD: &str = "demo-password";

const INDUSTRIES: [&str; 5] = ["SaaS", "Fintech", "Healthcare", "Logistics", "Retail"];
const TITLES: [&str; 5] = ["CEO", "CTO", "VP Sales", "Head of Ops", "Procurement Lead"];
const NOTES: [&str; 4] = [
    "Intro call went well; interested in the team plan.",
    "Asked for SOC 2 report and pricing for 50 seats.",
    "Champion is pushing internally, budget review next month.",
    "Sent proposal. Waiting on legal redlines.",
];

pub struct SeedDemo;
#[async_trait]
impl Task for SeedDemo {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "seed_demo".to_string(),
            detail: "Create a demo user plus companies, contacts, deals and notes".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &task::Vars) -> Result<()> {
        let n_companies: i64 = vars.cli_arg("companies").ok().and_then(|v| v.parse().ok()).unwrap_or(50);
        let n_contacts: i64 = vars.cli_arg("contacts").ok().and_then(|v| v.parse().ok()).unwrap_or(1000);

        if users::Model::find_by_email(&ctx.db, DEMO_EMAIL).await.is_err() {
            let user = users::Model::create_with_password(
                &ctx.db,
                &RegisterParams {
                    email: DEMO_EMAIL.to_string(),
                    password: DEMO_PASSWORD.to_string(),
                    name: "Demo User".to_string(),
                },
            )
            .await?;
            user.into_active_model().verified(&ctx.db).await?;
        }

        if companies::Entity::find().count(&ctx.db).await? > 0 {
            println!("demo data already present; skipping");
            return Ok(());
        }

        // Bulk inserts skip per-row `before_save` hooks; the data is known-valid.
        let txn = ctx.db.begin().await?;
        companies::Entity::insert_many((1..=n_companies).map(|i| companies::ActiveModel {
            name: Set(format!("Company {i}")),
            domain: Set(Some(format!("company{i}.example"))),
            industry: Set(Some(INDUSTRIES[(i as usize) % INDUSTRIES.len()].to_string())),
            ..Default::default()
        }))
        .exec(&txn)
        .await?;

        let contact_rows: Vec<_> = (1..=n_contacts)
            .map(|i| contacts::ActiveModel {
                first_name: Set(format!("First{i}")),
                last_name: Set(format!("Last{i}")),
                email: Set(format!("person{i}@example.com")),
                title: Set(Some(TITLES[(i as usize) % TITLES.len()].to_string())),
                company_id: Set(Some(((i - 1) % n_companies) + 1)),
                ..Default::default()
            })
            .collect();
        // SQLite caps bound parameters per statement, so insert in chunks.
        for chunk in contact_rows.chunks(200) {
            contacts::Entity::insert_many(chunk.to_vec()).exec(&txn).await?;
        }

        let n_deals = (n_contacts / 5).max(1);
        deals::Entity::insert_many((1..=n_deals).map(|i| {
            let contact_id = (i * 5 - 4).min(n_contacts);
            deals::ActiveModel {
                title: Set(format!("Deal {i}: {} seats", 10 + (i % 9) * 5)),
                amount_cents: Set(Some(500_000 + (i % 17) * 250_000)),
                stage: Set(STAGES[(i as usize) % STAGES.len()].to_string()),
                contact_id: Set(Some(contact_id)),
                company_id: Set(Some(((contact_id - 1) % n_companies) + 1)),
                ..Default::default()
            }
        }))
        .exec(&txn)
        .await?;

        notes::Entity::insert_many((1..=n_contacts.min(100)).flat_map(|i| {
            (0..2).map(move |k| notes::ActiveModel {
                body: Set(NOTES[((i + k) as usize) % NOTES.len()].to_string()),
                contact_id: Set(i),
                ..Default::default()
            })
        }))
        .exec(&txn)
        .await?;
        txn.commit().await?;

        println!(
            "seeded {n_companies} companies, {n_contacts} contacts, {n_deals} deals; login: {DEMO_EMAIL} / {DEMO_PASSWORD}"
        );
        Ok(())
    }
}
