#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use loco_rs::prelude::*;

use crate::{
    dtos::{
        common::{ApiError, Page},
        companies::{CompanyDto, CreateCompany, UpdateCompany},
    },
    models::_entities::companies::{ActiveModel, Column, Entity},
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
) -> Result<Json<Page<CompanyDto>>> {
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
        return Ok(not_found("company not found"));
    };
    Ok(Json(CompanyDto::from(model)).into_response())
}

#[debug_handler]
async fn create(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateCompany>,
) -> Result<Response> {
    let item = ActiveModel {
        name: Set(params.name),
        domain: Set(params.domain),
        industry: Set(params.industry),
        ..Default::default()
    };
    // Validate up front: a failure inside before_save surfaces as a 500 DbErr.
    item.validate().map_err(ModelError::from)?;
    let item = item.insert(&ctx.db).await?;

    Ok((StatusCode::CREATED, Json(CompanyDto::from(item))).into_response())
}

#[debug_handler]
async fn update(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<UpdateCompany>,
) -> Result<Response> {
    let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return Ok(not_found("company not found"));
    };

    let mut item = model.into_active_model();
    item.name = Set(params.name);
    item.domain = Set(params.domain);
    item.industry = Set(params.industry);
    item.validate().map_err(ModelError::from)?;
    let item = item.update(&ctx.db).await?;

    Ok(Json(CompanyDto::from(item)).into_response())
}

#[debug_handler]
async fn remove(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return Ok(not_found("company not found"));
    };

    model.into_active_model().delete(&ctx.db).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/companies")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", delete(remove))
}
