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

pub async fn get_package_name_by_app_id(pool: &Pool<MySql>, app_id: &String) -> Option<String> {
    let rs = sqlx::query("SELECT package_name FROM apps WHERE app_id=?")
        .bind(app_id)
        .fetch_one(pool).await;
    match rs {
        Ok(v) => {
            let a: String = v.get(0);
            Some(a)
        },
        Err(e) => {
            print!("get_package_name_by_app_id: {}", e);
            None
        }
    }
}

pub async fn set_umkey(pool: &Pool<MySql>, param: &ReqBindUmKey) -> i32 {
    let package_name = get_package_name_by_app_id(pool, &param.app_id).await;
    if let Some(package_name) = package_name {
        let rs = sqlx::query("UPDATE um_apps SET package_name=? WHERE appkey=?")
            .bind(&package_name)
            .bind(&param.appkey)
            .execute(pool).await;
        match rs {
            Ok(v) => {
                0
            },
            Err(e) => {
                println!("set_umkey err: {}", e);
                1
            }
        }
    } else {
        1
    }
    
}

pub async fn calc_ads_daily_release_reports_by_date(pool: &Pool<MySql>, date: &String) -> Option<Vec<AdsDailyReleaseReport>> {
    let rs = sqlx::query_as::<_, AdsDailyReleaseReport>("SELECT SUM(cost) as cost, CAST(SUM(active_count) as SIGNED) as active, SUM(attribution_income_iaa) as iaa, package_name, stat_datetime, country FROM reports WHERE stat_datetime = ? GROUP BY package_name, stat_datetime, country")
        .bind(&date)
        .fetch_all(pool).await;
    match rs {
        Ok(list) => {
            let mut ret: Vec<AdsDailyReleaseReport> = Vec::new();
            let mut map: HashMap<String, AdsDailyReleaseReport> = HashMap::new();
            for item in list {
                let key = format!("{}-{}", item.package_name, item.stat_datetime);
                // if item.package_name.eq("com.onlinepet.huawei") {
                //     println!("{} {} {}", item.package_name, item.country, item.cost);
                // }
                if !map.contains_key(&key) {
                    let vo = AdsDailyReleaseReport {
                        package_name: item.package_name.clone(),
                        stat_datetime: item.stat_datetime.clone(),
                        cost: item.cost,
                        active: item.active,
                        iaa: item.iaa,
                        country: String::from("ALL"),
                    };
                    map.insert(key, vo);
                } else {
                    let vo: &mut AdsDailyReleaseReport = map.get_mut(&key).unwrap();
                    vo.cost += item.cost;
                    vo.active += item.active;
                    vo.iaa += item.iaa;
                }
                ret.push(item);
            }

            for k in map {
                // if k.1.package_name.eq("com.onlinepet.huawei") {
                //     println!("total {} {} {}", &k.1.package_name, &k.1.country, &k.1.cost);
                // }
                // println!("total cost : {} - {}", advertiser_id, &k.1.cost);
                ret.push(k.1);
            }
            
            // list.append(vo);
            // Some(list)
            Some(ret)
        },
        Err(e) => {
            println!("get_ads_daily_release_reports_by_date err : {}", e);
            None
        }
    }
}

async fn is_release_report_exists(pool: &Pool<MySql>, package_name: &String, stat_datetime: &String, country: &String, record_datetime: &String) -> bool {
    let rs = sqlx::query("SELECT 1 FROM ads_daily_release_reports WHERE package_name=? AND stat_datetime=? AND country=? AND record_datetime=?")
        .bind(package_name)
        .bind(stat_datetime)
        .bind(country)
        .bind(record_datetime)
        .fetch_one(pool).await;
    match rs {
        Ok(v) => {
            let a: i32 = v.get(0);
            a == 1
        },
        Err(_) => false
    }
}

pub async fn insert_or_update_daily_release_report(pool: &Pool<MySql>, vo: &AdsDailyReleaseReport, record_date: &String) {
    // let today = Local::now().format("%Y-%m-%d").to_string();
    let is_exists = is_release_report_exists(pool, &vo.package_name, &vo.stat_datetime, &vo.country, record_date).await;
    if is_exists {
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
            Ok(v) => {},
            Err(e) => {
                println!("insert_or_update_daily_release_report err {}", e);
            }
        }
    } else {
        let rs = sqlx::query("INSERT INTO ads_daily_release_reports
            (package_name, cost, active, iaa, stat_datetime, record_datetime, country)
            VALUES(?,?,?,?,?,?,?) ON DUPLICATE KEY UPDATE cost=VALUES(cost),active=VALUES(active),iaa=VALUES(iaa);
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

    // let rs = sqlx::query("UPDATE ads_daily_release_reports
    // SET cost=?, active=?, iaa=? WHERE package_name=? AND stat_datetime=? AND record_datetime=? AND country=?")
    //     .bind(&vo.cost)
    //     .bind(&vo.active)
    //     .bind(&vo.iaa)
    //     .bind(&vo.package_name)
    //     .bind(&vo.stat_datetime)
    //     .bind(record_date)
    //     .bind(&vo.country)
    //     .execute(pool).await;

    // match rs {
    //     Ok(v) => {
    //         if v.rows_affected() == 0 {
    //             let rs = sqlx::query("INSERT INTO ads_daily_release_reports
    //             (package_name, cost, active, iaa, stat_datetime, record_datetime, country)
    //             VALUES(?,?,?,?,?,?,?);
    //             ")
    //                 .bind(&vo.package_name)
    //                 .bind(&vo.cost)
    //                 .bind(&vo.active)
    //                 .bind(&vo.iaa)
    //                 .bind(&vo.stat_datetime)
    //                 .bind(record_date)
    //                 .bind(&vo.country)
    //                 .execute(pool).await;
                
    //             match rs {
    //                 Ok(v) => {},
    //                 Err(e) => {
    //                     println!("insert ads_daily_release_reports err {}", e);
    //                 }
    //             }
    //         }
    //     },
    //     Err(e) => {
    //         println!("insert_or_update_daily_release_report err {}", e);
    //     }
    // }
}


pub async fn get_app_roas(pool: &Pool<MySql>, param: &ReqRoas) -> Option<Vec<AdsRoas>> {
    let sql = if let Some(country) = &param.country {
        format!("SELECT package_name, cost, active, iaa, DATE_FORMAT(stat_datetime, '%Y-%m-%d') as stat_datetime, DATE_FORMAT(record_datetime, '%Y-%m-%d') as record_datetime FROM ads_daily_release_reports WHERE package_name=? AND country='{}' AND stat_datetime BETWEEN ? AND ?", country)
    } else {
        "SELECT package_name, cost, active, iaa, DATE_FORMAT(stat_datetime, '%Y-%m-%d') as stat_datetime, DATE_FORMAT(record_datetime, '%Y-%m-%d') as record_datetime FROM ads_daily_release_reports WHERE package_name=? AND country='ALL' AND stat_datetime BETWEEN ? AND ?".to_string()
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

pub async fn add_app_gallery(pool: &Pool<MySql>,client_id: &String, client_secret: &String, connect_client_id: &Option<String>, connect_client_secret: &Option<String>, remark: &Option<String>) -> i32 {
    let rs = sqlx::query("INSERT INTO ads_account
        (client_id, client_secret, connect_client_id, connect_client_secret, remark)
        VALUES(?,?,?,?,?) ON DUPLICATE KEY UPDATE client_secret=VALUES(client_secret), connect_client_id=VALUES(connect_client_id), connect_client_secret=VALUES(connect_client_secret), remark=VALUES(remark);
        ")
            .bind(client_id)
            .bind(client_secret)
            .bind(connect_client_id)
            .bind(connect_client_secret)
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

pub async fn get_unbind_apps(pool: &Pool<MySql>) -> Option<Vec<App>> {
    let rs = sqlx::query_as::<_, App>("SELECT * FROM apps WHERE ISNULL(package_name)")
    .fetch_all(pool)
    .await;
    match rs {
        Ok(list) => Some(list),
        Err(v) => {
            println!("get_apps err: {}", v);
            None
        }
    }
}

pub async fn is_package_name_set(pool: &Pool<MySql>, package_name: &String) -> bool {
    let rs = sqlx::query("SELECT 1 FROM apps WHERE package_name=?")
            .bind(&package_name)
            .fetch_all(pool).await;
        
    match rs {
        Ok(v) => !v.is_empty(),
        Err(e) => {
            println!("is_package_name_set err {}", e);
            false
        }
    }
}

pub async fn save_unknown_package_name(pool: &Pool<MySql>, package_name: &str) {
    let rs = sqlx::query("INSERT INTO unknown_package_name SELECT ? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM unknown_package_name WHERE package_name=?)")
        .bind(package_name)
        .bind(package_name)
        .execute(pool).await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("save_unknown_package_name err: {}", e);
        }
    }
}

pub async fn get_expired_ads_token(pool: &Pool<MySql>) -> Option<Vec<AdsToken>> {
    let rs = sqlx::query_as::<_, AdsToken>("SELECT * from ads_account WHERE ISNULL(expire_time) OR expire_time < UNIX_TIMESTAMP()*1000+1800")
        .fetch_all(pool)
        .await;
    match rs {
        Ok(list) => Some(list),
        Err(e) => {
            println!("get_expired_ads_token err {}", e);
            None
        }
    }
}

pub async fn get_expired_connect_token(pool: &Pool<MySql>) -> Option<Vec<ConnectToken>> {
    let rs = sqlx::query_as::<_, ConnectToken>("SELECT * from ads_account WHERE NOT ISNULL(connect_client_id) AND (ISNULL(connect_expire_time) OR connect_expire_time < UNIX_TIMESTAMP()*1000+300)")
        .fetch_all(pool)
        .await;
    match rs {
        Ok(list) => Some(list),
        Err(e) => {
            println!("get_expired_connect_token err {}", e);
            None
        }
    }
}

pub async fn get_one_unknown_package_name(pool: &Pool<MySql>) -> Option<Vec<UnknownPackageName>> {
    let rs = sqlx::query_as::<_, UnknownPackageName>("SELECT * from unknown_package_name")
        .fetch_all(pool)
        .await;
    match rs {
        Ok(list) => Some(list),
        Err(e) => {
            println!("get_expired_connect_token err {}", e);
            None
        }
    }
}

pub async fn get_untry_connect_token(pool: &Pool<MySql>, package_name: &str) -> Option<Vec<ConnectToken>> {
    let rs = sqlx::query_as::<_, ConnectToken>("SELECT * FROM ads_account aa WHERE NOT ISNULL(aa.connect_client_id) AND aa.client_id NOT IN (SELECT client_id FROM unknown_package_name_except WHERE package_name=?)")
        .bind(package_name)
        .fetch_all(pool)
        .await;
    match rs {
        Ok(list) => Some(list),
        Err(e) => {
            println!("get_untry_connect_token err {}", e);
            None
        }
    }
    
}

pub async fn add_package_name_except_client_id(pool: &Pool<MySql>, connect_client_id: &str, package_name: &str) {
    let rs = sqlx::query("INSERT INTO unknown_package_name_except (package_name, client_id) SELECT ?,? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM unknown_package_name_except WHERE package_name=? AND client_id=?)")
        .bind(package_name)
        .bind(connect_client_id)
        .bind(package_name)
        .bind(connect_client_id)
        .execute(pool)
        .await;
    match rs {
        Ok(e) => {},
        Err(e) => {
            println!("add_package_name_except_client_id err {}", e);
        }
    }
}

pub async fn update_app_package_name(pool: &Pool<MySql>, app_id: &str, package_name: &str) {
    let rs = sqlx::query("UPDATE apps SET package_name=? WHERE app_id=?")
        .bind(package_name)
        .bind(app_id)
        .execute(pool)
        .await;
    match rs {
        Ok(e) => {},
        Err(e) => {
            println!("update_app_package_name err {}", e);
        }
    }
}

pub async fn remove_unknown_package_name(pool: &Pool<MySql>, package_name: &str) {
    let rs = sqlx::query("DELETE FROM unknown_package_name WHERE package_name=?")
        .bind(package_name)
        .execute(pool)
        .await;
    match rs {
        Ok(e) => {},
        Err(e) => {
            println!("remove_unknown_package_name err {}", e);
        }
    }
}

pub async fn save_um_apps(pool: &Pool<MySql>, appkey: &str, name: &str) {
    let rs = sqlx::query("INSERT INTO um_apps (appkey, `name`) VALUES (?,?) ON DUPLICATE KEY UPDATE `name`=VALUES(`name`)")
        .bind(appkey)
        .bind(name)
        .execute(pool)
        .await;
    match rs {
        Ok(e) => {},
        Err(e) => {
            println!("save_um_apps err {}", e);
        }
    }
}

pub async fn get_um_apps_with_package_name(pool: &Pool<MySql>) -> Option<Vec<UMApp>>  {
    let rs = sqlx::query_as::<_, UMApp>("SELECT * FROM um_apps WHERE NOT ISNULL(package_name)")
        .fetch_all(pool)
        .await;
    match rs {
        Ok(e) => Some(e),
        Err(e) => {
            println!("get_um_apps_with_package_name err {}", e);
            None
        }
    }
}

pub async fn save_app_umeng_retention(pool: &Pool<MySql>, appkey: &String, infos: &Vec<RetentionInfo>) {
    let mut sql = String::from("INSERT INTO um_retention (appkey, date, install, r1, r2, r3, r4, r5, r6, r7, r15, r30) VALUES ");
    let mut vs = vec![];
    for _ in infos {
        vs.push("(?,?,?,?,?,?,?,?,?,?,?,?)");
    }

    sql = sql + vs.join(",").as_str();
    sql += " ON DUPLICATE KEY UPDATE install=VALUES(install), r1=VALUES(r1), r2=VALUES(r2), r3=VALUES(r3), r4=VALUES(r4), r5=VALUES(r5), r6=VALUES(r6), r7=VALUES(r7), r15=VALUES(r15), r30=VALUES(r30)";
    

    let mut query = sqlx::query(sql.as_str());
    for info in infos {
        let r1 = info.retentionRate.get(0);
        let r2 = info.retentionRate.get(1);
        let r3 = info.retentionRate.get(2);
        let r4 = info.retentionRate.get(3);
        let r5 = info.retentionRate.get(4);
        let r6 = info.retentionRate.get(5);
        let r7 = info.retentionRate.get(6);
        let r15 = info.retentionRate.get(7);
        let r30 = info.retentionRate.get(8);
        query = query.bind(appkey)
            .bind(&info.date)
            .bind(&info.totalInstallUser)
            .bind(r1)
            .bind(r2)
            .bind(r3)
            .bind(r4)
            .bind(r5)
            .bind(r6)
            .bind(r7)
            .bind(r15)
            .bind(r30);
    };

    let rs = query.execute(pool)
        .await;
    match rs {
        Ok(e) => {},
        Err(e) => {
            println!("save_um_apps err {}", e);
        }
    }
}

pub async fn save_app_umeng_duration(pool: &Pool<MySql>, appkey: &str, date: &str, average: i32) {
    let rs = sqlx::query("INSERT INTO um_retention (appkey, date, duration) VALUES (?,?,?) ON DUPLICATE KEY UPDATE duration=VALUES(duration)")
        .bind(appkey)
        .bind(date)
        .bind(average)
        .execute(pool)
        .await;
    match rs {
        Ok(e) => {},
        Err(e) => {
            println!("save_app_umeng_duration err {}", e);
        }
    }
}

pub async fn get_umeng_app_without_duration(pool: &Pool<MySql>) -> Option<Vec<UMRetentionApp>> {
    let rs = sqlx::query_as::<_, UMRetentionApp>("SELECT appkey, `date` FROM um_retention WHERE ISNULL(duration) OR duration=0")
        .fetch_all(pool)
        .await;
    match rs {
        Ok(e) => Some(e),
        Err(e) => {
            println!("save_app_umeng_duration err {}", e);
            None
        }
    }
}
