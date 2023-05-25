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
use sqlx::{Pool, MySql, mysql::MySqlPoolOptions};
use std::{time::Duration, fs};
use ctrl::game_controller;

use crate::{ctrl::game_service::GameService, model::MysqlConfig};


#[get("/azadmin/test")]
pub async fn test() -> impl Responder {
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
                "root",server_config.password,"127.0.0.1","3306");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("start");
    let server_config: ServerConfig = ServerConfig::read();


    let pool = MySqlPoolOptions::new()
    .max_connections(30)
    .connect(&get_mysql_connect_url(&server_config))
    .await.unwrap_or_else(|_| { std::process::exit(0) });

    if server_config.profiles.eq("prod") {
        println!("prod");
        actix_rt::spawn(async move {
            let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&get_mysql_connect_url(&server_config))
            .await.unwrap_or_else(|_| { std::process::exit(0) });
            let game_service = GameService::create();
    
            let mut interval = time::interval(Duration::from_secs(60));
            let mut task_interval_1 = 1;
            loop {
                interval.tick().await;
                task_interval_1 = task_interval_1 - 1;
                if task_interval_1 == 0 {
                    task_interval_1 = 5;
                    game_controller::check_access_token(&pool, &game_service).await;
                    game_controller::query_reports(&pool, &game_service).await;
                    game_controller::query_ads_reports(&pool, &game_service).await;
                    game_controller::query_last_90_day_earning_reports(&pool, &game_service).await;
                    game_controller::query_last_90_release_reports(&pool, &game_service).await;
                    game_controller::query_umeng_apps(&pool, &game_service).await;
                    game_controller::query_last_30_umeng_retentions(&pool, &game_service).await;
                    game_controller::query_umeng_duration(&pool, &game_service).await;
                }
                game_controller::check_package_app_id(&pool, &game_service).await
            }
        });
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
            .service(game_controller::authcallback)
            .service(game_controller::get_advertisers)
            .service(game_controller::add_advertiser)
            .service(game_controller::get_app_gallery)
            .service(game_controller::add_app_gallery)
            .service(game_controller::get_reports)
            .service(game_controller::get_earnings_reports)
            .service(game_controller::download_reports)
            .service(game_controller::get_sum_reports)
            .service(game_controller::get_apps)
            .service(game_controller::get_ads_accounts)
            .service(game_controller::bind_app)
            .service(game_controller::set_umkey)
            .service(game_controller::get_app_roas)
            .service(game_controller::upload_app_icon)
            .service(game_controller::icon)
            .service(game_controller::get_countries)
            .service(game_controller::login_admin)
            .service(test)
    })
    .bind(("0.0.0.0", 13491))?
    .run()
    .await
}


