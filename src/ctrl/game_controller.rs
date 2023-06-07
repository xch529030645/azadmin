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
use crate::{AppState, ctrl::{game_service::GameService}, model::*, lib::req::AuthorizationCode, user_data::UserData};

use super::game_repository;


#[get("/azadmin/auth")]
pub async fn authcallback(data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Query<AuthorizationCode>) -> impl Responder {
    game_service.authcallback(&data.pool, &param.0).await;
    return Results::err(0)
}

#[get("/azadmin/auth_webhook")]
pub async fn authcallback_webhook(data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Query<AuthorizationCode>) -> impl Responder {
    game_service.authcallback_webhook(&data.pool, &param.0).await;
    return Results::err(0)
}

#[post("/azadmin/login_admin")]
async fn login_admin(data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqLogin>) -> impl Responder {
    let jwt = game_service.login_admin(&data.pool, &param.0).await;
    Results::done(&jwt.as_ref())
}

#[post("/azadmin/change_password")]
async fn change_password(data: web::Data<AppState>, user_data: UserData, game_service: web::Data<GameService>, param: web::Json<ReqChangePassword>) -> impl Responder {
    let err = game_service.change_password(&data.pool, user_data.id, &param.0).await;
    Results::err(err)
}

#[post("/azadmin/get_privileges")]
async fn get_privileges(user_data: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>) -> impl Responder {
    let jwt = game_service.get_privileges(&data.pool, &user_data).await;
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

#[post("/azadmin/update_advertiser_remark")]
pub async fn update_advertiser_remark(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqAdvRemark>) -> impl Responder {
    let err = game_service.update_advertiser_remark(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/save_admin")]
pub async fn save_admin(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<Admin>) -> impl Responder {
    let err = game_service.save_admin(&data.pool, &param.0).await;
    return Results::err(err);
}

#[post("/azadmin/get_admin")]
pub async fn get_admin(user_data: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>) -> impl Responder {
    let list = game_service.get_admin(&data.pool, user_data.id).await;
    return Results::done(&list.as_ref());
}

#[post("/azadmin/get_admin_advertisers")]
pub async fn get_admin_advertisers(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>) -> impl Responder {
    let list = game_service.get_admin_advertisers(&data.pool).await;
    return Results::done(&list.as_ref());
}

#[post("/azadmin/save_admin_advertisers")]
pub async fn save_admin_advertisers(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqSaveAdminAdvertiser>) -> impl Responder {
    let err = game_service.save_admin_advertisers(&data.pool, &param.0).await;
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
pub async fn get_reports(user_data: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqQueryReports>) -> impl Responder {
    let p = if user_data.id == 4 {
        let mut param = param.0;
        param.package_name = Some("com.craftsman.go.huawei".to_string());
        param
    } else {
        param.0
    };
    let list = game_service.get_reports(&data.pool, &p).await;
    return Results::done(&list.as_ref());
}

#[post("/azadmin/get_earnings_reports")]
pub async fn get_earnings_reports(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqQueryEarningReports>) -> impl Responder {
    let list = game_service.get_earnings_reports(&data.pool, &param.0).await;
    return Results::done(&list.as_ref());
}

#[get("/azadmin/download_reports")]
pub async fn download_reports(data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Query<ReqQueryReports>) -> Result<HttpResponse> {
    // println!("download_reports");
    let query = &param.0;

    let ret = game_service.generate_reports_csv(&data.pool, query).await;

    let filename = format!("{}-{}-{}.csv", &query.country.clone().unwrap_or("".to_string()), &query.start_date.clone().unwrap_or("".to_string()), &query.end_date.clone().unwrap_or("".to_string()));
    let cd = ContentDisposition {
        disposition: DispositionType::FormData,
        parameters: vec![
            DispositionParam::Name(String::from("upload")),
            DispositionParam::Filename(filename),
        ],
    };
    
    let mut builder = HttpResponse::Ok();
    builder.insert_header((actix_web::http::header::CONTENT_DISPOSITION, cd));

    Ok(builder.body(ret))
}

#[post("/azadmin/get_sum_reports")]
pub async fn get_sum_reports(user_data: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqQueryReports>) -> impl Responder {
    let p = if user_data.id == 4 {
        let mut param = param.0;
        param.package_name = Some("com.craftsman.go.huawei".to_string());
        param
    } else {
        param.0
    };
    let rs = game_service.get_sum_reports(&data.pool, &p).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/get_overview")]
pub async fn get_overview(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Json<ReqQueryOverview>) -> impl Responder {
    let rs = game_service.get_overview(&data.pool, &param.0).await;
    return Results::done(&rs.as_ref());
}

#[post("/azadmin/get_apps")]
pub async fn get_apps(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>) -> impl Responder {
    let list = game_service.get_apps(&data.pool).await;
    return Results::done(&list.as_ref());
}

#[post("/azadmin/get_ads_accounts")]
pub async fn get_ads_accounts(_: UserData, data: web::Data<AppState>, game_service: web::Data<GameService>) -> impl Responder {
    let list = game_service.get_ads_accounts(&data.pool).await;
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

pub async fn query_reports(pool: &Pool<MySql>, game_service: &GameService) {
    // println!("query_reports");
    let s = game_service.clone();
    let p = pool.clone();
    actix_rt::spawn(async move {
        s.query_reports(&p, &Local::now(), &Local::now()).await;
    });
    // game_service.query_reports(&pool, &Local::now(), &Local::now()).await;

}

pub async fn query_ads_reports(pool: &Pool<MySql>, game_service:&GameService) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    game_service.query_ads_reports(pool, &today).await;
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
    let time = timestamp() % 86400;
    // println!("query_last_30_umeng_retentions {}", time);
    if time >= 3600 {
        game_service.query_last_30_umeng_retentions(pool).await;
    }
}

pub async fn query_umeng_duration(pool: &Pool<MySql>, game_service: &GameService) {
    let time = timestamp() % 86400;
    // println!("query_umeng_duration {}", time);
    if time >= 7200 {
        game_service.query_umeng_duration(pool).await;
    }
}

fn timestamp() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() as i64
}

pub async fn query_last_90_day_earning_reports(pool: &Pool<MySql>, game_service: &GameService) {
    let time = timestamp() % 86400;
    println!("query_last_day_earning_reports {}", time);
    if time >= 3600 {
        game_service.query_last_90_day_earning_reports(pool).await;
    }
    
}

pub async fn restart_mysql(pool: &Pool<MySql>) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let is_executed = game_repository::is_daily_task_executed(pool, &today, 6).await;
    if !is_executed {
        let th = Command::new("/usr/bin/systemctl").arg("restart mysqld").spawn();
        match th {
            Ok(th) => {
                let out = th.wait_with_output();
                match out {
                    Ok(output) => {
                        let out = String::from_utf8(output.stdout).unwrap();
                        println!("{}", out);
                        game_repository::done_daily_query_task(pool, &today).await;
                    }
                    Err(e) => {
                        println!("restart_mysql {}", e);
                    }
                }
                
            }
            Err(e) => {
                println!("restart_mysql {}", e);
            }
        }
        
    }
}