use axum::http::{HeaderName, HeaderValue};
use axum_test::multipart::{MultipartForm, Part};
use loco_crm::{app::App, models::contacts};
use loco_rs::testing::prelude::*;
use serial_test::serial;

use crate::support::{create_crm, htmx, mail_count, session_cookie};

#[tokio::test]
#[serial]
async fn anonymous_visitors_are_sent_to_login() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/dashboard").await;
        assert_eq!(res.status_code(), 303);
        assert_eq!(res.header("location"), "/login");

        // HTMX requests get an HX-Redirect instead of a swapped-in login page.
        let (k, v) = htmx();
        let res = request.get("/contacts").add_header(k, v).await;
        assert_eq!(res.status_code(), 401);
        assert_eq!(res.header("hx-redirect"), "/login");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn login_form_sets_http_only_cookie() {
    request::<App, _, _>(|request, ctx| async move {
        session_cookie(&request, &ctx).await;
        let res = request
            .post("/login")
            .form(&[("email", "tess@example.com"), ("password", "pass1234")])
            .await;
        assert_eq!(res.status_code(), 303);
        let cookie = res.header("set-cookie");
        let cookie = cookie.to_str().unwrap();
        assert!(
            cookie.starts_with("auth_token=")
                && cookie.contains("HttpOnly")
                && cookie.contains("SameSite=Lax")
        );

        let bad = request
            .post("/login")
            .form(&[("email", "tess@example.com"), ("password", "nope")])
            .await;
        assert_eq!(bad.status_code(), 401);
        assert!(bad.text().contains("Invalid email or password"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn signed_in_user_sees_dashboard_and_contacts() {
    request::<App, _, _>(|request, ctx| async move {
        let crm = create_crm(&ctx).await;
        let (k, v) = session_cookie(&request, &ctx).await;

        let dash = request
            .get("/dashboard")
            .add_header(k.clone(), v.clone())
            .await;
        assert_eq!(dash.status_code(), 200);
        assert!(
            dash.text().contains("$12,500"),
            "pipeline value should render"
        );
        assert!(dash.headers().contains_key("server-timing"));

        let list = request
            .get("/contacts?q=ada")
            .add_header(k.clone(), v.clone())
            .await;
        assert!(list.text().contains("Ada Lovelace") && list.text().contains("Acme Corp"));

        // Live search asks for the rows partial only.
        let (hk, hv) = htmx();
        let rows = request
            .get("/contacts?q=nobody")
            .add_header(k.clone(), v.clone())
            .add_header(hk, hv)
            .add_header(
                HeaderName::from_static("hx-target"),
                HeaderValue::from_static("contact-rows"),
            )
            .await;
        assert!(rows.text().contains("No contacts match") && !rows.text().contains("<html"));

        let show = request
            .get(&format!("/contacts/{}", crm.contact.id))
            .add_header(k, v)
            .await;
        assert!(show.text().contains("Acme rollout"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn contact_form_validates_on_the_model() {
    request::<App, _, _>(|request, ctx| async move {
        let (k, v) = session_cookie(&request, &ctx).await;
        let (hk, hv) = htmx();
        let res = request
            .post("/contacts")
            .add_header(k.clone(), v.clone())
            .add_header(hk.clone(), hv.clone())
            .form(&[("first_name", "Bob"), ("email", "not-an-email")])
            .await;
        assert_eq!(res.status_code(), 422);
        assert!(res.text().contains("valid email"), "{}", res.text());

        let ok = request
            .post("/contacts")
            .add_header(k, v)
            .add_header(hk, hv)
            .form(&[
                ("first_name", "Bob"),
                ("email", "bob@example.com"),
                ("company_id", ""),
            ])
            .await;
        assert_eq!(ok.status_code(), 200);
        assert!(ok
            .header("hx-redirect")
            .to_str()
            .unwrap()
            .starts_with("/contacts/"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn notes_and_ai_summary() {
    request::<App, _, _>(|request, ctx| async move {
        let crm = create_crm(&ctx).await;
        let (k, v) = session_cookie(&request, &ctx).await;
        let id = crm.contact.id;

        let note = request
            .post(&format!("/contacts/{id}/notes"))
            .add_header(k.clone(), v.clone())
            .form(&[("body", "Wants a pilot in Q1")])
            .await;
        assert!(note.text().contains("Wants a pilot in Q1"));

        // ForegroundBlocking in tests: the worker runs inline with the mock LLM.
        let summary = request
            .post(&format!("/contacts/{id}/summarize"))
            .add_header(k.clone(), v.clone())
            .await;
        assert!(summary.text().contains("[mock]"), "{}", summary.text());
        let saved = contacts::Model::find_by_id(&ctx.db, id).await.unwrap();
        assert!(saved.ai_summary.unwrap().contains("1 note"));

        let draft = request
            .post(&format!("/contacts/{id}/draft-email"))
            .add_header(k, v)
            .await;
        assert!(draft.text().contains("Hi Ada Lovelace"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn winning_a_deal_emails_the_sales_inbox() {
    request::<App, _, _>(|request, ctx| async move {
        let crm = create_crm(&ctx).await;
        let (k, v) = session_cookie(&request, &ctx).await;
        let before = mail_count(&ctx);

        let res = request
            .post(&format!("/deals/{}/stage", crm.deal.id))
            .add_header(k.clone(), v.clone())
            .form(&[("stage", "won")])
            .await;
        assert_eq!(res.status_code(), 200);
        assert_eq!(mail_count(&ctx), before + 1);
        let mail = ctx
            .mailer
            .as_ref()
            .unwrap()
            .deliveries()
            .messages
            .last()
            .cloned()
            .unwrap();
        assert!(mail.contains("Deal won: Acme rollout"));

        let bad = request
            .post(&format!("/deals/{}/stage", crm.deal.id))
            .add_header(k, v)
            .form(&[("stage", "bogus")])
            .await;
        assert_eq!(
            bad.status_code(),
            400,
            "unknown stage is a model validation error"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn avatar_upload_round_trips_through_storage() {
    request::<App, _, _>(|request, ctx| async move {
        let crm = create_crm(&ctx).await;
        let (k, v) = session_cookie(&request, &ctx).await;
        let png: &[u8] = b"\x89PNG\r\n\x1a\nfake";

        let form = MultipartForm::new().add_part(
            "avatar",
            Part::bytes(png.to_vec())
                .file_name("a.png")
                .mime_type("image/png"),
        );
        let res = request
            .post(&format!("/contacts/{}/avatar", crm.contact.id))
            .add_header(k.clone(), v.clone())
            .multipart(form)
            .await;
        assert_eq!(res.status_code(), 303);

        let key = contacts::Model::find_by_id(&ctx.db, crm.contact.id)
            .await
            .unwrap()
            .avatar_key
            .unwrap();
        let file = request
            .get(&format!("/files/{key}"))
            .add_header(k.clone(), v.clone())
            .await;
        assert_eq!(file.header("content-type"), "image/png");
        assert_eq!(file.as_bytes().as_ref(), png);

        let wrong = MultipartForm::new().add_part(
            "avatar",
            Part::bytes(b"hi".to_vec())
                .file_name("a.txt")
                .mime_type("text/plain"),
        );
        let res = request
            .post(&format!("/contacts/{}/avatar", crm.contact.id))
            .add_header(k, v)
            .multipart(wrong)
            .await;
        assert_eq!(res.status_code(), 400);
    })
    .await;
}
