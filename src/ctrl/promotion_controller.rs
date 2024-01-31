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

#[post("/azadmin/query_position_detail")]
pub async fn query_position_detail(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<ReqQueryPositionDetail>) -> impl Responder {
    let rs = promotion_service.query_position_detail(&data.pool, &param.0).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/query_assets")]
pub async fn query_assets(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<FormQueryAssets>) -> impl Responder {
    let rs = promotion_service.query_assets(&data.pool, &param.0).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/get_collection_tasks")]
pub async fn get_collection_tasks(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>) -> impl Responder {
    let rs = promotion_service.get_collection_tasks(&data.pool).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/update_collection_tasks")]
pub async fn update_collection_tasks(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<FormUpdateCollectionStatus>) -> impl Responder {
    let err = promotion_service.update_collection_tasks(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/update_collection_advertisers")]
pub async fn update_collection_advertisers(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<FormUpdateCollectionAdvertisers>) -> impl Responder {
    let err = promotion_service.update_collection_advertisers(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/get_collection_operations")]
pub async fn get_collection_operations(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<FormCollectionId>) -> impl Responder {
    let rs = promotion_service.get_collection_operations(&data.pool, &param.0).await;
    return Results::done(&rs.as_ref());
}


pub async fn fetch_assets(pool: &Pool<MySql>, promotion_service: &PromotionService) {
    println!("start fetch_assets");
    promotion_service.fetch_assets(pool).await;
}


#[post("/azadmin/query_position_price")]
pub async fn query_position_price(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<ReqWebQueryPositionPrice>) -> impl Responder {
    let rs = promotion_service.query_position_price(&data.pool, &param.0).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/create_ads")]
pub async fn create_ads(user_data: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<ReqCreateAds>) -> impl Responder {
    promotion_service.create_ads(&data.pool, &user_data, &param.0).await
}

#[post("/azadmin/search_assets")]
pub async fn search_assets(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<ReqSearchAssets>) -> impl Responder {
    let rs = promotion_service.search_assets(&data.pool, &param.0).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/add_collection")]
pub async fn add_collection(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<ReqSaveCollection>) -> impl Responder {
    let err = promotion_service.add_collection(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/del_collection")]
pub async fn del_collection(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<ReqDelCollection>) -> impl Responder {
    let err = promotion_service.del_collection(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/get_ads")]
pub async fn get_ads(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<ReqGetAds>) -> impl Responder {
    let rs = promotion_service.get_ads(&data.pool, &param.0).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/save_budget_plan")]
pub async fn save_budget_plan(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<ReqSaveBudgetPlan>) -> impl Responder {
    let err = promotion_service.save_budget_plan(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/get_budget_plans")]
pub async fn get_budget_plans(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>) -> impl Responder {
    let rs = promotion_service.get_budget_plans(&data.pool).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/delete_budget_plan")]
pub async fn delete_budget_plan(_: UserData, data: web::Data<AppState>, promotion_service: web::Data<PromotionService>, param: web::Json<ReqDeleteBudgetPlan>) -> impl Responder {
    let err = promotion_service.delete_budget_plan(&data.pool, &param.0).await;
    return Results::err(err);
}
