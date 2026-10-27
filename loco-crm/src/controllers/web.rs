//! Server-rendered CRM (Tera + HTMX). Loco 1.x no longer generates HTML
//! scaffolds, so these handlers and `assets/views/**` are hand-written.
#![allow(clippy::unused_async)]
use axum::{
    extract::{FromRequestParts, Multipart},
    http::{header, request::Parts, HeaderMap, StatusCode},
    response::Redirect,
};
use loco_rs::{environment::Environment, prelude::*};
use serde::Deserialize;
use serde_json::json;
use std::path::Path as FsPath;

use crate::{
    llm,
    mailers::auth::AuthMailer,
    models::{
        _entities::{companies as companies_entity, contacts as contacts_entity, deals as deals_entity},
        companies,
        contacts::{self, NewContact},
        deals, notes,
        users::{self, LoginParams, RegisterParams},
    },
    views::crm::{ColumnView, ContactView, DealView, NoteView, StageView, UserView},
    workers::{ai_summarize, deal_won},
};

const AUTH_COOKIE: &str = "auth_token";
const PER_PAGE: u64 = 20;

/// The logged-in user for HTML routes. Like `auth::JWTWithUser`, but a missing
/// or bad token redirects to `/login` (or sends `HX-Redirect` for HTMX calls)
/// instead of returning a JSON 401.
pub struct CurrentUser(pub users::Model);

impl FromRequestParts<AppContext> for CurrentUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, ctx: &AppContext) -> Result<Self, Response> {
        match auth::JWTWithUser::<users::Model>::from_request_parts(parts, ctx).await {
            Ok(a) => Ok(Self(a.user)),
            Err(_) if parts.headers.contains_key("hx-request") => {
                Err((StatusCode::UNAUTHORIZED, [("hx-redirect", "/login")]).into_response())
            }
            Err(_) => Err(Redirect::to("/login").into_response()),
        }
    }
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}

/// Full redirect for normal requests, `HX-Redirect` for HTMX ones.
fn redirect_to(headers: &HeaderMap, to: &str) -> Result<Response> {
    if is_htmx(headers) {
        format::render().header("hx-redirect", to).empty()
    } else {
        format::redirect(to)
    }
}

fn auth_cookie(ctx: &AppContext, token: &str, max_age: u64) -> String {
    let secure = if ctx.environment == Environment::Production {
        "; Secure"
    } else {
        ""
    };
    format!("{AUTH_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}")
}

fn sign_in(ctx: &AppContext, user: &users::Model) -> Result<Response> {
    let jwt = ctx.config.get_jwt_config()?;
    let token = user
        .generate_jwt(&jwt.secret, jwt.expiration)
        .or_else(|_| unauthorized("unauthorized!"))?;
    format::render()
        .header(header::SET_COOKIE, auth_cookie(ctx, &token, jwt.expiration))
        .redirect("/dashboard")
}

// ---------- auth pages ----------

#[debug_handler]
async fn login_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "auth/login.html", json!({ "user": null, "error": null, "email": "" }))
}

#[debug_handler]
async fn login_submit(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Form(params): Form<LoginParams>,
) -> Result<Response> {
    match users::Model::find_by_email(&ctx.db, &params.email).await {
        Ok(user) if user.verify_password(&params.password) => sign_in(&ctx, &user),
        _ => format::render().status(StatusCode::UNAUTHORIZED).view(
            &v,
            "auth/login.html",
            json!({ "user": null, "error": "Invalid email or password.", "email": params.email }),
        ),
    }
}

#[debug_handler]
async fn register_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "auth/register.html",
        json!({ "user": null, "error": null, "name": "", "email": "" }),
    )
}

#[debug_handler]
async fn register_submit(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Form(params): Form<RegisterParams>,
) -> Result<Response> {
    let user = match users::Model::create_with_password(&ctx.db, &params).await {
        Ok(user) => user,
        Err(err) => {
            return format::render().status(StatusCode::UNPROCESSABLE_ENTITY).view(
                &v,
                "auth/register.html",
                json!({ "user": null, "error": err.to_string(), "name": params.name, "email": params.email }),
            )
        }
    };
    let user = user
        .into_active_model()
        .set_email_verification_sent(&ctx.db)
        .await?;
    AuthMailer::send_welcome(&ctx, &user).await?;
    sign_in(&ctx, &user)
}

#[debug_handler]
async fn logout(State(ctx): State<AppContext>) -> Result<Response> {
    format::render()
        .header(header::SET_COOKIE, auth_cookie(&ctx, "", 0))
        .redirect("/login")
}

// ---------- dashboard ----------

#[debug_handler]
async fn dashboard(
    CurrentUser(user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let pipeline = deals::Model::pipeline_summary(&ctx.db).await?;
    let recent = notes::Model::recent(&ctx.db, 8).await?;
    let stats = json!({
        "contacts": contacts_entity::Entity::find().count(&ctx.db).await?,
        "companies": companies_entity::Entity::find().count(&ctx.db).await?,
        "deals": deals_entity::Entity::find().count(&ctx.db).await?,
    });
    format::render().view(
        &v,
        "dashboard/index.html",
        json!({
            "user": UserView::from(&user),
            "stats": stats,
            "pipeline": pipeline.iter().map(StageView::from).collect::<Vec<_>>(),
            "recent": recent.iter().map(NoteView::from).collect::<Vec<_>>(),
        }),
    )
}

// ---------- contacts ----------

#[derive(Debug, Deserialize)]
struct ListQuery {
    q: Option<String>,
    page: Option<u64>,
}

#[debug_handler]
async fn contacts_index(
    CurrentUser(user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response> {
    let page = query.page.unwrap_or(1).max(1);
    let q = query.q.unwrap_or_default();
    let (rows, pages) = contacts::Model::search(&ctx.db, Some(&q), page, PER_PAGE).await?;
    let data = json!({
        "user": UserView::from(&user),
        "contacts": rows.iter().map(ContactView::from).collect::<Vec<_>>(),
        "q": q,
        "page": page,
        "pages": pages,
        "companies": companies::Model::all_by_name(&ctx.db).await?,
        "error": null,
    });
    // Live search swaps only the table body.
    let template = if headers.get("hx-target").is_some_and(|t| t == "contact-rows") {
        "contacts/_rows.html"
    } else {
        "contacts/index.html"
    };
    format::render().view(&v, template, data)
}

#[debug_handler]
async fn contacts_create(
    CurrentUser(_user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    headers: HeaderMap,
    Form(form): Form<NewContact>,
) -> Result<Response> {
    match contacts::Model::create_from_form(&ctx.db, form).await {
        Ok(contact) => redirect_to(&headers, &format!("/contacts/{}", contact.id)),
        Err(ModelError::Validation(errors)) => format::render()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .view(
                &v,
                "contacts/_form.html",
                json!({
                    "error": ModelError::Validation(errors).to_string(),
                    "companies": companies::Model::all_by_name(&ctx.db).await?,
                }),
            ),
        Err(e) => Err(e.into()),
    }
}

#[debug_handler]
async fn contacts_show(
    CurrentUser(user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let d = contacts::Model::find_details(&ctx.db, id).await?;
    let company = d.company.as_ref();
    format::render().view(
        &v,
        "contacts/show.html",
        json!({
            "user": UserView::from(&user),
            "contact": ContactView::new(&d.contact, company),
            "notes": d.notes.iter().map(NoteView::from).collect::<Vec<_>>(),
            "deals": d.deals.iter().map(|deal| DealView::new(deal, company)).collect::<Vec<_>>(),
            "pending": false,
        }),
    )
}

#[derive(Debug, Deserialize)]
struct NoteForm {
    body: String,
}

#[debug_handler]
async fn notes_create(
    CurrentUser(_user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i64>,
    Form(form): Form<NoteForm>,
) -> Result<Response> {
    let contact = contacts::Model::find_by_id(&ctx.db, id).await?;
    match notes::Model::create_for_contact(&ctx.db, contact.id, &form.body).await {
        Ok(note) => format::render().view(&v, "contacts/_note.html", json!({ "note": NoteView::from(&note) })),
        Err(ModelError::Validation(e)) => format::render()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .html(&format!("<p class=\"error\">{}</p>", ModelError::Validation(e))),
        Err(e) => Err(e.into()),
    }
}

/// Clear the summary and queue the AI worker; the partial polls until it lands.
#[debug_handler]
async fn summarize(
    CurrentUser(_user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let contact = contacts::Model::find_by_id(&ctx.db, id).await?;
    let contact = contact.into_active_model().set_ai_summary(&ctx.db, None).await?;
    ai_summarize::Worker::perform_later(&ctx, ai_summarize::WorkerArgs { contact_id: contact.id }).await?;
    // In ForegroundBlocking mode (tests) the job already ran; re-read either way.
    let contact = contacts::Model::find_by_id(&ctx.db, id).await?;
    render_summary(&v, &contact)
}

#[debug_handler]
async fn summary(
    CurrentUser(_user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let contact = contacts::Model::find_by_id(&ctx.db, id).await?;
    render_summary(&v, &contact)
}

fn render_summary(v: &TeraView, contact: &contacts::Model) -> Result<Response> {
    format::render().view(
        v,
        "contacts/_summary.html",
        json!({
            "contact": ContactView::new(contact, None),
            "pending": contact.ai_summary.is_none(),
        }),
    )
}

/// Synchronous LLM call, for comparison with the queued summary.
#[debug_handler]
async fn draft_email(
    CurrentUser(user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let contact = contacts::Model::find_by_id(&ctx.db, id).await?;
    let dossier = contact.dossier(&ctx.db).await?;
    let system = format!(
        "You write short, friendly sales follow-up emails. Sign off as {}. \
         Plain text, under 120 words, no subject line.",
        user.name
    );
    let completion = llm::complete(&ctx, &system, &dossier).await?;
    format::render().view(&v, "contacts/_draft.html", json!({ "draft": completion }))
}

#[debug_handler]
async fn upload_avatar(
    CurrentUser(_user): CurrentUser,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    mut multipart: Multipart,
) -> Result<Response> {
    let contact = contacts::Model::find_by_id(&ctx.db, id).await?;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?
    {
        if field.name() != Some("avatar") {
            continue;
        }
        let ext = match field.content_type() {
            Some("image/png") => "png",
            Some("image/jpeg") => "jpg",
            Some("image/gif") => "gif",
            Some("image/webp") => "webp",
            _ => return bad_request("avatar must be a PNG, JPEG, GIF or WebP image"),
        };
        let bytes = field
            .bytes()
            .await
            .map_err(|e| Error::BadRequest(e.to_string()))?;
        let key = format!("avatars/{}-{}.{ext}", contact.id, Uuid::new_v4());
        ctx.storage.upload(FsPath::new(&key), &bytes).await?;
        contact.into_active_model().set_avatar(&ctx.db, key).await?;
        return redirect_to(&headers, &format!("/contacts/{id}"));
    }
    bad_request("no avatar file in form")
}

/// Serve uploaded files from `ctx.storage` (local disk here, S3 in production).
#[debug_handler]
async fn file(
    CurrentUser(_user): CurrentUser,
    State(ctx): State<AppContext>,
    Path(key): Path<String>,
) -> Result<Response> {
    if !key.starts_with("avatars/") || key.contains("..") {
        return not_found();
    }
    let bytes: Vec<u8> = ctx.storage.download(FsPath::new(&key)).await?;
    let mime = match key.rsplit('.').next() {
        Some("png") => "image/png",
        Some("jpg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };
    Ok(([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "private, max-age=3600")], bytes).into_response())
}

// ---------- deals board ----------

const CARDS_PER_COLUMN: u64 = 25;

#[debug_handler]
async fn deals_board(
    CurrentUser(user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let board = deals::Model::board(&ctx.db, CARDS_PER_COLUMN).await?;
    format::render().view(
        &v,
        "deals/index.html",
        json!({
            "user": UserView::from(&user),
            "columns": board.iter().map(ColumnView::from).collect::<Vec<_>>(),
            "stages": deals::STAGES,
        }),
    )
}

#[derive(Debug, Deserialize)]
struct StageForm {
    stage: String,
}

#[debug_handler]
async fn deals_move(
    CurrentUser(_user): CurrentUser,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i64>,
    Form(form): Form<StageForm>,
) -> Result<Response> {
    let deal = deals::Model::find_by_id(&ctx.db, id).await?;
    let was_won = deal.stage == "won";
    let deal = deal.into_active_model().move_to_stage(&ctx.db, &form.stage).await?;
    if deal.stage == "won" && !was_won {
        deal_won::Worker::perform_later(&ctx, deal_won::WorkerArgs { deal_id: deal.id }).await?;
    }
    let board = deals::Model::board(&ctx.db, CARDS_PER_COLUMN).await?;
    format::render().view(
        &v,
        "deals/_board.html",
        json!({
            "columns": board.iter().map(ColumnView::from).collect::<Vec<_>>(),
            "stages": deals::STAGES,
        }),
    )
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/login", get(login_page))
        .add("/login", post(login_submit))
        .add("/register", get(register_page))
        .add("/register", post(register_submit))
        .add("/logout", post(logout))
        .add("/dashboard", get(dashboard))
        .add("/contacts", get(contacts_index))
        .add("/contacts", post(contacts_create))
        .add("/contacts/{id}", get(contacts_show))
        .add("/contacts/{id}/notes", post(notes_create))
        .add("/contacts/{id}/summarize", post(summarize))
        .add("/contacts/{id}/summary", get(summary))
        .add("/contacts/{id}/draft-email", post(draft_email))
        .add("/contacts/{id}/avatar", post(upload_avatar))
        .add("/files/{*key}", get(file))
        .add("/deals", get(deals_board))
        .add("/deals/{id}/stage", post(deals_move))
}
