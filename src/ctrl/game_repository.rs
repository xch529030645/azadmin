use std::{time::{SystemTime, UNIX_EPOCH}, collections::HashMap, vec};
use chrono::{Local, DateTime, Days};
use sqlx::{Pool, MySql, Row};

use crate::model::*;

// use crate::{lib::{server_api, req::{AuthorizationCode}, response::*}, model::*, auth};

pub async fn is_daily_query_task_executed(pool: &Pool<MySql>, date: &String) -> bool {
    let rs = sqlx::query("SELECT 1 FROM task_daily_query WHERE date=?")
        .bind(date)
        .fetch_one(pool).await;
    match rs {
        Ok(v) => true,
        Err(e) => false
    }
}

pub async fn done_daily_query_task(pool: &Pool<MySql>, date: &String) {
    let rs = sqlx::query("INSERT INTO task_daily_query (date) VALUES (?)")
        .bind(date)
        .execute(pool).await;
    match rs {
        Ok(v) => {

        },
        Err(e) => {
            println!("done_daily_query_task err: {}", e);
        }
    }
}

pub async fn bind_app(pool: &Pool<MySql>, param: &ReqBindApp) -> i32 {
    let rs = sqlx::query("UPDATE apps SET package_name=? WHERE app_id=?")
        .bind(&param.package_name)
        .bind(&param.app_id)
        .execute(pool).await;
    match rs {
        Ok(v) => {
            0
        },
        Err(e) => {
            println!("bind_app err: {}", e);
            1
        }
    }
}

pub async fn calc_ads_daily_release_reports_by_date(pool: &Pool<MySql>, date: &String) -> Option<Vec<AdsDailyReleaseReport>> {
    let rs = sqlx::query_as::<_, AdsDailyReleaseReport>("SELECT SUM(cost) as cost, CAST(SUM(active_count) as SIGNED) as active, SUM(attribution_income_iaa) as iaa, package_name, stat_datetime, country FROM reports WHERE stat_datetime = ? GROUP BY package_name, stat_datetime, country")
        .bind(&date)
        .fetch_all(pool).await;
    match rs {
        Ok(list) => Some(list),
        Err(e) => {
            println!("get_ads_daily_release_reports_by_date err : {}", e);
            None
        }
    }
}

pub async fn insert_or_update_daily_release_report(pool: &Pool<MySql>, vo: &AdsDailyReleaseReport, record_date: &String) {
    // let today = Local::now().format("%Y-%m-%d").to_string();
    let rs = sqlx::query("UPDATE ads_daily_release_reports
    SET cost=?, active=?, iaa=? WHERE package_name=? AND stat_datetime=? AND record_datetime=? AND country=?")
        .bind(&vo.cost)
        .bind(&vo.active)
        .bind(&vo.iaa)
        .bind(&vo.package_name)
        .bind(&vo.stat_datetime)
        .bind(record_date)
        .bind(&vo.country)
        .execute(pool).await;

    match rs {
        Ok(v) => {
            if v.rows_affected() == 0 {
                let rs = sqlx::query("INSERT INTO ads_daily_release_reports
                (package_name, cost, active, iaa, stat_datetime, record_datetime, country)
                VALUES(?,?,?,?,?,?,?);
                ")
                    .bind(&vo.package_name)
                    .bind(&vo.cost)
                    .bind(&vo.active)
                    .bind(&vo.iaa)
                    .bind(&vo.stat_datetime)
                    .bind(record_date)
                    .bind(&vo.country)
                    .execute(pool).await;
                
                match rs {
                    Ok(v) => {},
                    Err(e) => {
                        println!("insert ads_daily_release_reports err {}", e);
                    }
                }
            }
        },
        Err(e) => {
            println!("insert_or_update_daily_release_report err {}", e);
        }
    }
}


pub async fn get_app_roas(pool: &Pool<MySql>, param: &ReqRoas) -> Option<Vec<AdsRoas>> {
    let sql = if let Some(country) = &param.country {
        format!("SELECT package_name, SUM(cost) as cost, CAST(SUM(active) AS SIGNED) as active, SUM(iaa) as iaa, DATE_FORMAT(stat_datetime, '%Y-%m-%d') as stat_datetime, DATE_FORMAT(record_datetime, '%Y-%m-%d') as record_datetime FROM ads_daily_release_reports WHERE package_name=? AND country='{}' AND stat_datetime BETWEEN ? AND ? GROUP BY package_name,stat_datetime,record_datetime", country)
    } else {
        "SELECT package_name, SUM(cost) as cost, CAST(SUM(active) AS SIGNED) as active, SUM(iaa) as iaa, DATE_FORMAT(stat_datetime, '%Y-%m-%d') as stat_datetime, DATE_FORMAT(record_datetime, '%Y-%m-%d') as record_datetime FROM ads_daily_release_reports WHERE package_name=? AND stat_datetime BETWEEN ? AND ? GROUP BY package_name,stat_datetime,record_datetime".to_string()
    };
    let rs = sqlx::query_as::<_, AdsRoas>(sql.as_str())
        .bind(&param.package_name)
        .bind(&param.start_date)
        .bind(&param.end_date)
        .fetch_all(pool).await;
    match rs {
        Ok(list) => Some(list),
        Err(e) => {
            println!("get_ads_daily_release_reports_by_date err : {}", e);
            None
        }
    }
}

pub async fn save_app_icon(pool: &Pool<MySql>, app_id: &String, icon: &String) {
    let rs = sqlx::query("UPDATE apps SET icon=? WHERE app_id=?")
        .bind(icon)
        .bind(app_id)
        .execute(pool).await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("save_app_icon err: {}", e);
        }
    }
}

pub async fn add_advertiser(pool: &Pool<MySql>, advertiser_id: &String, client_id: &str, client_secret: &str, redirect_url: &str) -> i32 {
    let rs = sqlx::query("INSERT INTO advertisers
        (advertiser_id, client_id, secret, create_time, update_time)
        VALUES(?,?,?,NOW(),NOW());
        ")
            .bind(advertiser_id)
            .bind(client_id)
            .bind(client_secret)
            .execute(pool).await;
        
    match rs {
        Ok(v) => 0,
        Err(e) => {
            println!("insert add_advertiser err {}", e);
            1
        }
    }
}

pub async fn add_app_gallery(pool: &Pool<MySql>,client_id: &String, client_secret: &String, remark: &Option<String>) -> i32 {
    let rs = sqlx::query("INSERT INTO ads_account
        (client_id, client_secret, remark)
        VALUES(?,?,?) ON DUPLICATE KEY UPDATE client_secret=VALUES(client_secret), remark=VALUES(remark);
        ")
            .bind(client_id)
            .bind(client_secret)
            .bind(remark)
            .execute(pool).await;
        
    match rs {
        Ok(v) => 0,
        Err(e) => {
            println!("insert add_advertiser err {}", e);
            1
        }
    }
}


pub async fn save_country(pool: &Pool<MySql>, name: String) {
    let rs = sqlx::query("INSERT INTO countries (name) SELECT ? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM countries WHERE name=?)")
            .bind(&name)
            .bind(&name)
            .execute(pool).await;
        
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("save_country err {}", e);
        }
    }
}

pub async fn get_countries(pool: &Pool<MySql>) -> Option<Vec<Country>> {
    let rs = sqlx::query_as::<_, Country>("SELECT * FROM countries")
        .fetch_all(pool).await;
    match rs {
        Ok(list) => Some(list),
        Err(e) => {
            println!("get_ads_daily_release_reports_by_date err : {}", e);
            None
        }
    }
}