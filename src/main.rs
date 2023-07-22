mod model;
mod auth;
mod user_data;
// mod game_service;
mod ctrl;
mod lib;

use actix_cors::Cors;
use actix_rt::time;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use model::ServerConfig;
use sqlx::{Pool, MySql, mysql::MySqlPoolOptions, Executor};
use std::{time::Duration, fs};
use ctrl::game_controller;

use crate::{ctrl::{game_service::GameService, promotion_service::PromotionService, promotion_controller}, model::MysqlConfig, lib::server_api};


#[get("/azadmin/test")]
pub async fn test() -> impl Responder {
    // let bytes = std::fs::read(path).unwrap();  // Vec<u8>
    // let hash = sha256::digest_bytes(&bytes);
    HttpResponse::Ok().body("ok")
}

#[derive(Clone)]
pub struct AppState {
    pool: Pool<MySql>
}


// #[get("/azadmin/auth")]
// async fn authcallback(data: web::Data<AppState>, game_service: web::Data<GameService>, param: web::Query<AuthorizationCode>) -> impl Responder {

//    return Results::err(0)
// }


fn get_mysql_connect_url(server_config: &ServerConfig) -> String {
    return format!("mysql://{}:{}@{}:{}/azadmin?useUnicode=true&characterEncoding=utf8&serverTimezone=Asia/Shanghai&useOldAliasMetadataBehavior=true",
                "root",server_config.password,server_config.host,server_config.port);
}

fn start_timer(server_config: ServerConfig) {
    actix_rt::spawn(async move {
        let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&get_mysql_connect_url(&server_config))
        .await.unwrap_or_else(|_| { std::process::exit(0) });
        let game_service = GameService::create();
        // let promotion_service = PromotionService::create();

        let mut interval = time::interval(Duration::from_secs(60));
        let mut task_interval_1 = 1;
        loop {
            interval.tick().await;
            task_interval_1 = task_interval_1 - 1;
            if task_interval_1 == 0 {
                task_interval_1 = 5;
                game_controller::restart_mysql(&pool).await;
                game_controller::check_access_token(&pool, &game_service).await;
                game_controller::query_campaigns(&pool, &game_service).await;
                game_controller::query_reports(&pool, &game_service).await;
                game_controller::query_ads_reports(&pool, &game_service).await;
                game_controller::query_last_90_day_earning_reports(&pool, &game_service).await;
                game_controller::query_last_90_release_reports(&pool, &game_service).await;
                game_controller::query_umeng_apps(&pool, &game_service).await;
                game_controller::query_last_30_umeng_retentions(&pool, &game_service).await;
                game_controller::query_umeng_duration(&pool, &game_service).await;

                // promotion_controller::fetch_assets(&pool, &promotion_service).await;
            }
            game_controller::check_package_app_id(&pool, &game_service).await
        }
    });
}

fn start_ad_thread(server_config: ServerConfig) {
    actix_rt::spawn(async move {
        let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&get_mysql_connect_url(&server_config))
        .await.unwrap_or_else(|_| { std::process::exit(0) });
        let promotion_service = PromotionService::create();

        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            promotion_service.check_create_ads(&pool).await;
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("start");
    let server_config: ServerConfig = ServerConfig::read();


    let pool = MySqlPoolOptions::new()
    .after_connect(|conn, _meta| Box::pin(async move {
        conn.execute("set time_zone = '+8:00';").await;
        Ok(())
    }))
    .max_connections(30)
    .connect(&get_mysql_connect_url(&server_config))
    .await.unwrap_or_else(|_| { std::process::exit(0) });

    println!("start mysql");


    if server_config.profiles.eq("prod") {
        println!("prod");
        start_timer(server_config.clone());
        start_ad_thread(server_config.clone());
    }
    

    // 客户端ID: 108128867
    // 密钥: 9fae7c0bf88cd122d3fc87d1709fba4326d1f157a5d82a6aa1b4cf7123c09d54
    // https://dat.311419.cn/azadmin/auth
    // 


    HttpServer::new(move || {
        let cors = Cors::default()
              .allow_any_origin()
              .allow_any_header()
              .allow_any_method()
              .max_age(3600);
        
        App::new()
            .wrap(cors)
            // .wrap(auth::Auth)
            .app_data(web::Data::new(AppState {
                pool: pool.clone(),
            }))
            .app_data(web::Data::new(GameService::create()))
            .app_data(web::Data::new(PromotionService::create()))
            .service(game_controller::authcallback)
            .service(game_controller::authcallback_webhook)
            .service(game_controller::get_advertisers)
            .service(game_controller::add_advertiser)
            .service(game_controller::get_app_gallery)
            .service(game_controller::add_app_gallery)
            .service(game_controller::get_reports)
            .service(game_controller::get_earnings_reports)
            .service(game_controller::download_reports)
            .service(game_controller::get_sum_reports)
            .service(game_controller::get_overview)
            .service(game_controller::get_apps)
            .service(game_controller::get_ads_accounts)
            .service(game_controller::bind_app)
            .service(game_controller::set_umkey)
            .service(game_controller::get_app_roas)
            .service(game_controller::upload_app_icon)
            .service(game_controller::icon)
            .service(game_controller::get_countries)
            .service(game_controller::login_admin)
            .service(game_controller::change_password)
            .service(game_controller::get_privileges)
            .service(game_controller::update_advertiser_remark)
            .service(game_controller::save_admin)
            .service(game_controller::get_admin)
            .service(game_controller::get_admin_advertisers)
            .service(game_controller::save_admin_advertisers)
            .service(promotion_controller::create_audience_package)
            .service(promotion_controller::sync_audience_package)
            .service(promotion_controller::get_audience_package)
            .service(promotion_controller::get_position)
            .service(promotion_controller::query_position_detail)
            .service(promotion_controller::query_assets)
            .service(promotion_controller::get_collection_tasks)
            .service(promotion_controller::update_collection_tasks)
            .service(promotion_controller::update_collection_advertisers)
            .service(promotion_controller::get_collection_operations)
            .service(promotion_controller::query_position_price)
            .service(promotion_controller::create_ads)
            .service(promotion_controller::search_assets)
            .service(promotion_controller::add_collection)
            .service(promotion_controller::del_collection)
            .service(test)
    })
    .bind(("0.0.0.0", 13491))?
    .run()
    .await
}


