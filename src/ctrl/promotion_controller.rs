use std::{fs, io::Write, time::{SystemTime, UNIX_EPOCH}, process::Command};

use actix_files::NamedFile;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest, Result, http::header::{ContentDisposition, DispositionType, DispositionParam}};
use chrono::Local;
use sqlx::{Pool, MySql, mysql::MySqlPoolOptions};
use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
    },
    Multipart,
};
use crate::{AppState, ctrl::{promotion_service::PromotionService}, model::*, lib::req::AuthorizationCode, user_data::UserData};


#[post("/azadmin/create_audience_package")]
pub async fn create_audience_package(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<FormCreateAudience>) -> impl Responder {
    let err = promotion_service.create_audience_package(&data.pool, param.0).await;
    return Results::err(err);
}

#[post("/azadmin/sync_audience_package")]
pub async fn sync_audience_package(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<FormAdvertiser>) -> impl Responder {
    let err = promotion_service.sync_audience_package(&data.pool, &param.0.advertiser_id).await;
    return Results::err(err);
}

#[post("/azadmin/get_audience_package")]
pub async fn get_audience_package(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<FormAdvertiser>) -> impl Responder {
    let rs = promotion_service.get_audience_package(&data.pool, &param.0.advertiser_id).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/get_position")]
pub async fn get_position(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<FormAdvertiser>) -> impl Responder {
    let rs = promotion_service.get_position(&data.pool, &param.0.advertiser_id).await;
    return Results::done(&rs.as_ref());
}

pub async fn fetch_assets(pool: &Pool<MySql>, promotion_service: &PromotionService) {
    println!("start fetch_assets");
    promotion_service.fetch_assets(pool).await;
}
