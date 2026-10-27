#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use loco_rs::prelude::*;

use crate::{
    dtos::{
        common::{ApiError, Page},
        contacts::{ContactDto, CreateContact, UpdateContact},
    },
    models::_entities::contacts::{ActiveModel, Column, Entity},
};

/// Build a 404 response shaped as the [`ApiError`] envelope.
fn not_found(message: &str) -> Response {
    let error = ApiError {
        code: "not_found".to_string(),
        message: message.to_string(),
        details: None,
    };
    (StatusCode::NOT_FOUND, Json(error)).into_response()
}

#[debug_handler]
async fn list(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(pagination): Query<query::PaginationQuery>,
) -> Result<Json<Page<ContactDto>>> {
    let res = query::paginate(
        &ctx.db,
        Entity::find().order_by_asc(Column::Id),
        None,
        &pagination,
    )
    .await?;

    Ok(Json(Page::from_query(res)))
}

#[debug_handler]
async fn get_one(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return Ok(not_found("contact not found"));
    };
    Ok(Json(ContactDto::from(model)).into_response())
}

#[debug_handler]
async fn create(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateContact>,
) -> Result<Response> {
    let item = ActiveModel {
        first_name: Set(params.first_name),
        last_name: Set(params.last_name),
        email: Set(params.email),
        phone: Set(params.phone),
        title: Set(params.title),
        ai_summary: Set(params.ai_summary),
        avatar_key: Set(params.avatar_key),
        company_id: Set(params.company_id),
        ..Default::default()
    };
    // Validate up front: a failure inside before_save surfaces as a 500 DbErr.
    item.validate().map_err(ModelError::from)?;
    let item = item.insert(&ctx.db).await?;

    Ok((StatusCode::CREATED, Json(ContactDto::from(item))).into_response())
}

#[debug_handler]
async fn update(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<UpdateContact>,
) -> Result<Response> {
    let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return Ok(not_found("contact not found"));
    };

    let mut item = model.into_active_model();
    item.first_name = Set(params.first_name);
    item.last_name = Set(params.last_name);
    item.email = Set(params.email);
    item.phone = Set(params.phone);
    item.title = Set(params.title);
    item.ai_summary = Set(params.ai_summary);
    item.avatar_key = Set(params.avatar_key);
    item.company_id = Set(params.company_id);
    item.validate().map_err(ModelError::from)?;
    let item = item.update(&ctx.db).await?;

    Ok(Json(ContactDto::from(item)).into_response())
}

#[debug_handler]
async fn remove(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return Ok(not_found("contact not found"));
    };

    model.into_active_model().delete(&ctx.db).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/contacts")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", delete(remove))
}
