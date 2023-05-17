use std::{fs, io::Write};

use actix_files::NamedFile;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest, Result};
use chrono::Local;
use sqlx::{Pool, MySql};
use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
    },
    Multipart,
};
use crate::{AppState, ctrl::{game_service::GameService}, model::*, lib::req::AuthorizationCode, user_data::UserData};


#[get("/azadmin/auth")]
pub async fn authcallback(data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Query<AuthorizationCode>) -> impl Responder {
    game_service.authcallback(&data.pool, &param.0).await;
    return Results::err(0)
}

#[post("/azadmin/login_admin")]
async fn login_admin(data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqLogin>) -> impl Responder {
    let jwt = game_service.login_admin(&data.pool, &param.0).await;
    Results::done(&jwt.as_ref())
}

#[post("/azadmin/advertisers")]
pub async fn get_advertisers(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>) -> impl Responder {
    let list = game_service.get_advertisers(&data.pool).await;
    return Results::done(&list.as_ref());
}

#[post("/azadmin/add_advertiser")]
pub async fn add_advertiser(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqAddAdv>) -> impl Responder {
    let err = game_service.add_advertiser(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/get_app_gallery")]
pub async fn get_app_gallery(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>) -> impl Responder {
    let list = game_service.get_app_gallery(&data.pool).await;
    return Results::done(&list.as_ref());
}

#[post("/azadmin/add_app_gallery")]
pub async fn add_app_gallery(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqAddGallery>) -> impl Responder {
    let err = game_service.add_app_gallery(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/get_reports")]
pub async fn get_reports(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqQueryReports>) -> impl Responder {
    let list = game_service.get_reports(&data.pool, &param.0).await;
    return Results::done(&list.as_ref());
}

#[post("/azadmin/get_sum_reports")]
pub async fn get_sum_reports(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqQueryReports>) -> impl Responder {
    let rs = game_service.get_sum_reports(&data.pool, &param.0).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/get_apps")]
pub async fn get_apps(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>) -> impl Responder {
    let list = game_service.get_apps(&data.pool).await;
    return Results::done(&list.as_ref());
}

#[post("/azadmin/bind_app")]
pub async fn bind_app(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqBindApp>) -> impl Responder {
    let err = game_service.bind_app(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/set_umkey")]
pub async fn set_umkey(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqBindUmKey>) -> impl Responder {
    let err = game_service.set_umkey(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/get_app_roas")]
pub async fn get_app_roas(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqRoas>) -> impl Responder {
    let rs = game_service.get_app_roas(&data.pool, &param.0).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/get_countries")]
async fn get_countries(data: web::Data<AppState>, game_service: web::Data<GameService>) -> impl Responder {
    let rs = game_service.get_countries(&data.pool).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/upload_app_icon")]
pub async fn upload_app_icon(data: web::Data<AppState>, game_service: web::Data<GameService>, MultipartForm(form): MultipartForm<UploadForm>, param: web::Query<ReqUploadAppIcon>) -> impl Responder {
    let dir = std::path::Path::new("./icons");
    if !dir.exists() {
        fs::create_dir(dir);
    }

    for f in form.files {
        let local_path = "./icons/".to_string() + &param.0.app_id+".png";
        let rs = fs::copy(f.file.path(), std::path::Path::new(local_path.as_str()));
        match rs {
            Ok(v) => { 
                println!("save icon success")
            },
            Err(e) => {
                println!("save icon failed {}", e)
            }
        }
    }

    // f.file.persist(local_path).unwrap();
    
    let png = format!("https://dat.311419.cn/azadmin/icon/{}.png", &param.0.app_id);
    game_service.save_app_icon(&data.pool, &param.0.app_id, &png).await;
    return Results::done(&Some(png).as_ref());
}

#[get("/azadmin/icon/{filename:.*}")]
async fn icon(filename: web::Path<String>) -> Result<NamedFile> {
    Ok(NamedFile::open(format!("icons/{}", filename))?)
}



pub async fn check_access_token(pool: &Pool<MySql>, game_service:&GameService) {
    game_service.check_access_token(pool).await;
}

pub async fn query_reports(pool: &Pool<MySql>, game_service:&GameService) {
    game_service.query_reports(pool, &Local::now(), &Local::now()).await;
}

pub async fn query_ads_reports(pool: &Pool<MySql>, game_service:&GameService) {
    game_service.query_ads_reports(pool).await;
}

pub async fn query_last_90_release_reports(pool: &Pool<MySql>, game_service:&GameService) {
    game_service.query_last_90_release_reports(pool).await;
}

pub async fn check_package_app_id(pool: &Pool<MySql>, game_service: &GameService) {
    game_service.check_package_app_id(pool).await;
}

pub async fn query_umeng_apps(pool: &Pool<MySql>, game_service: &GameService) {
    game_service.query_umeng_apps(pool).await;
}

pub async fn query_last_30_umeng_retentions(pool: &Pool<MySql>, game_service: &GameService) {
    let time = Local::now().timestamp() % 86400;
    if time >= 32400 && time < 32750 {
        game_service.query_last_30_umeng_retentions(pool).await;
    }
}

pub async fn query_umeng_duration(pool: &Pool<MySql>, game_service: &GameService) {
    game_service.query_umeng_duration(pool).await;
}