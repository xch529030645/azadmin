use std::{time::{SystemTime, UNIX_EPOCH}, collections::HashMap, vec};
use chrono::{Local, DateTime, Days};
use sqlx::{Pool, MySql, Row, Execute};

use crate::{model::*, lib::response::{ResReportVo, ResQueryAssets}};

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
                        country: String::from("ALL")
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

pub async fn calc_ads_daily_release_reports_group_by_advertiser_by_date(pool: &Pool<MySql>, date: &String) -> Option<Vec<AdsDailyReleaseReportAdv>> {
    let rs = sqlx::query_as::<_, AdsDailyReleaseReportAdv>("SELECT SUM(cost) as cost, CAST(SUM(active_count) as SIGNED) as active, SUM(attribution_income_iaa) as iaa, package_name, stat_datetime, country, advertiser_id FROM reports WHERE stat_datetime = ? GROUP BY package_name, stat_datetime, country, advertiser_id")
    .bind(&date)
    .fetch_all(pool).await;
    match rs {
        Ok(list) => {
            let mut ret: Vec<AdsDailyReleaseReportAdv> = Vec::new();
            let mut map: HashMap<String, AdsDailyReleaseReportAdv> = HashMap::new();
            for item in list {
                let key = format!("{}-{}-{:?}", item.package_name, item.stat_datetime, item.advertiser_id);
                // if item.package_name.eq("com.onlinepet.huawei") {
                //     println!("{} {} {}", item.package_name, item.country, item.cost);
                // }
                if !map.contains_key(&key) {
                    let vo = AdsDailyReleaseReportAdv {
                        package_name: item.package_name.clone(),
                        stat_datetime: item.stat_datetime.clone(),
                        cost: item.cost,
                        active: item.active,
                        iaa: item.iaa,
                        country: String::from("ALL"),
                        advertiser_id: item.advertiser_id.clone()
                    };
                    map.insert(key, vo);
                } else {
                    let vo: &mut AdsDailyReleaseReportAdv = map.get_mut(&key).unwrap();
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
            println!("calc_ads_daily_release_reports_group_by_advertiser_by_date err : {}", e);
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
    let country = if let Some(country) = &param.country {
        country.as_str()
    } else {
        "ALL"
    };

    let sql = format!("SELECT a.package_name, a.cost, a.active, a.iaa, DATE_FORMAT(a.stat_datetime, '%Y-%m-%d') as stat_datetime, DATE_FORMAT(a.record_datetime, '%Y-%m-%d') as record_datetime, b.earnings FROM ads_advertiser_daily_release_reports a 
    LEFT JOIN apps c ON a.package_name = c.package_name 
    LEFT JOIN ads_daily_earnings_reports b ON a.stat_datetime=b.stat_datetime AND b.app_id = c.app_id 
    WHERE a.package_name=? AND a.country=? AND a.stat_datetime BETWEEN ? AND ?");

    // let sql = if let Some(country) = &param.country {
    //     format!("SELECT package_name, cost, active, iaa, DATE_FORMAT(stat_datetime, '%Y-%m-%d') as stat_datetime, DATE_FORMAT(record_datetime, '%Y-%m-%d') as record_datetime FROM ads_daily_release_reports WHERE package_name=? AND country='{}' AND stat_datetime BETWEEN ? AND ?", country)
    // } else {
    //     "SELECT package_name, cost, active, iaa, DATE_FORMAT(stat_datetime, '%Y-%m-%d') as stat_datetime, DATE_FORMAT(record_datetime, '%Y-%m-%d') as record_datetime FROM ads_daily_release_reports WHERE package_name=? AND country='ALL' AND stat_datetime BETWEEN ? AND ?".to_string()
    // };
    let rs = sqlx::query_as::<_, AdsRoas>(sql.as_str())
        .bind(&param.package_name)
        .bind(country)
        .bind(&param.start_date)
        .bind(&param.end_date)
        .fetch_all(pool).await;
    match rs {
        Ok(list) => Some(list),
        Err(e) => {
            println!("get_app_roas err : {}", e);
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

pub async fn update_advertiser_remark(pool: &Pool<MySql>, param: &ReqAdvRemark) -> i32 {
    let rs = sqlx::query("UPDATE advertisers SET remark=? WHERE advertiser_id=?")
        .bind(&param.remark)
        .bind(&param.advertiser_id)
        .execute(pool).await;
        
    match rs {
        Ok(v) => 0,
        Err(e) => {
            println!("update_advertiser_remark err {}", e);
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
            println!("get_countries err : {}", e);
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
    let rs = sqlx::query_as::<_, AdsToken>("SELECT * from ads_account WHERE ISNULL(expire_time) OR expire_time < UNIX_TIMESTAMP()*1000+3600")
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
    let yesterday = Local::now().checked_sub_days(Days::new(1)).unwrap().format("%Y-%m-%d").to_string();

    let rs = sqlx::query_as::<_, UMRetentionApp>("SELECT appkey, `date` FROM um_retention WHERE date=? AND (ISNULL(duration) OR duration=0)")
        .bind(yesterday)
        .fetch_all(pool)
        .await;
    match rs {
        Ok(e) => Some(e),
        Err(e) => {
            println!("get_umeng_app_without_duration err {}", e);
            None
        }
    }
}

pub async fn is_daily_task_executed(pool: &Pool<MySql>, today: &String, task_type: i32) -> bool {
    let rs = sqlx::query("SELECT 1 FROM task_daily_records WHERE `date`=? AND task_type=?")
        .bind(today)
        .bind(task_type)
        .fetch_one(pool).await;
    match rs {
        Ok(v) => {
            let a: bool = v.get(0);
            a
        },
        Err(e) => {
            println!("is_daily_task_executed: {}", e);
            false
        }
    }
}

pub async fn execute_daily_task_done(pool: &Pool<MySql>, today: &String, task_type: i32) {
    let rs = sqlx::query("INSERT INTO task_daily_records (`date`,task_type) SELECT ?,? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM task_daily_records WHERE `date`=? AND task_type=?)")
        .bind(today)
        .bind(task_type) 
        .bind(today)
        .bind(task_type)
        .execute(pool).await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            print!("execute_daily_task_done: {}", e);
        }
    }
}

pub async fn get_ads_accounts(pool: &Pool<MySql>) -> Option<Vec<AdsAccount>> {
    let rs = sqlx::query_as::<_, AdsAccount>("SELECT client_id, remark FROM ads_account")
        .fetch_all(pool)
        .await;
        match rs {
            Ok(list) => Some(list),
            Err(v) => {
                println!("get_ads_accounts err: {}", v);
                None
            }
        }
}

pub async fn get_ads_access_token(pool: &Pool<MySql>, client_id: &String) -> Option<String> {
    let rs = sqlx::query("SELECT access_token FROM ads_account WHERE client_id=?")
        .bind(client_id)
        .fetch_one(pool).await;
    match rs {
        Ok(v) => {
            let a: String = v.get(0);
            Some(a)
        },
        Err(e) => {
            print!("get_ads_access_token: {}", e);
            None
        }
    }
}

pub async fn get_marketing_access_token(pool: &Pool<MySql>, advertiser_id: &String) -> Option<String> {
    let rs = sqlx::query("SELECT access_token FROM advertisers WHERE advertiser_id=?")
        .bind(advertiser_id)
        .fetch_one(pool).await;
    match rs {
        Ok(v) => {
            let a: String = v.get(0);
            Some(a)
        },
        Err(e) => {
            print!("get_marketing_access_token: {}", e);
            None
        }
    }
}

pub async fn save_marketing_reports(pool: &Pool<MySql>, advertiser_id: &String, list: &Vec<ResReportVo>)  {
    let mut placeholders: Vec<&str> = vec![];
    for _ in list {
        placeholders.push("(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)");
    }

    let mut sql = "INSERT INTO azadmin.reports
        (advertiser_id, adgroup_id, adgroup_name, campaign_id, campaign_name, package_name, stat_datetime, `show_count`, click_count, cpc, thousand_show_cost, cost, download_count, download_cost, install_count, install_cost, active_count, active_cost, register_count, register_cost, retain_count, retain_cost, three_day_retain_count, three_day_retain_cost, subscribe_count, subscribe_cost, seven_day_retain_count, seven_day_retain_cost, publisher_real_price_one_day, ad_income_one_day_ltv_hms, ad_income_two_day_ltv_hms, ad_income_three_day_ltv_hms, ad_income_seven_day_ltv_hms, ad_income_fifteen_day_ltv_hms, ad_income_thirty_day_ltv_hms, ad_income_one_day_roi, ad_income_two_day_roi, ad_income_three_day_roi, ad_income_seven_day_roi, ad_income_fifteen_day_roi, ad_income_thirty_day_roi, attribution_income_iaa, attribution_income_iap_normalized, ad_position_id, country)
        VALUES ".to_string();
    sql += placeholders.join(",").as_str();
    sql += " ON DUPLICATE KEY UPDATE `show_count`=VALUES(`show_count`),
    click_count=VALUES(click_count),
    cpc=VALUES(cpc),
    thousand_show_cost=VALUES(thousand_show_cost),
    cost=VALUES(cost),
    download_count=VALUES(download_count),
    download_cost=VALUES(download_cost),
    install_count=VALUES(install_count),
    install_cost=VALUES(install_cost),
    active_count=VALUES(active_count),
    active_cost=VALUES(active_cost),
    register_count=VALUES(register_count),
    register_cost=VALUES(register_cost),
    retain_count=VALUES(retain_count),
    retain_cost=VALUES(retain_cost),
    three_day_retain_count=VALUES(three_day_retain_count),
    three_day_retain_cost=VALUES(three_day_retain_cost),
    subscribe_count=VALUES(subscribe_count),
    subscribe_cost=VALUES(subscribe_cost),
    seven_day_retain_count=VALUES(seven_day_retain_count),
    seven_day_retain_cost=VALUES(seven_day_retain_cost),
    publisher_real_price_one_day=VALUES(publisher_real_price_one_day),
    ad_income_one_day_ltv_hms=VALUES(ad_income_one_day_ltv_hms),
    ad_income_two_day_ltv_hms=VALUES(ad_income_two_day_ltv_hms),
    ad_income_three_day_ltv_hms=VALUES(ad_income_three_day_ltv_hms),
    ad_income_seven_day_ltv_hms=VALUES(ad_income_seven_day_ltv_hms),
    ad_income_fifteen_day_ltv_hms=VALUES(ad_income_fifteen_day_ltv_hms),
    ad_income_thirty_day_ltv_hms=VALUES(ad_income_thirty_day_ltv_hms),
    ad_income_one_day_roi=VALUES(ad_income_one_day_roi),
    ad_income_two_day_roi=VALUES(ad_income_two_day_roi),
    ad_income_three_day_roi=VALUES(ad_income_three_day_roi),
    ad_income_seven_day_roi=VALUES(ad_income_seven_day_roi),
    ad_income_fifteen_day_roi=VALUES(ad_income_fifteen_day_roi),
    ad_income_thirty_day_roi=VALUES(ad_income_thirty_day_roi),
    attribution_income_iaa=VALUES(attribution_income_iaa),
    attribution_income_iap_normalized=VALUES(attribution_income_iap_normalized)";

    let mut query = sqlx::query(sql.as_str());

    for vo in list {
        let stat_datetime = "".to_string() + &vo.stat_datetime[0..4] + "-" + &vo.stat_datetime[4..6] + "-" + &vo.stat_datetime[6..8];
        query = query.bind(advertiser_id)
        .bind(&vo.adgroup_id)
        .bind(&vo.adgroup_name)
        .bind(&vo.campaign_id)
        .bind(&vo.campaign_name)
        .bind(&vo.package_name)
        .bind(stat_datetime)
        .bind(&vo.show_count)
        .bind(&vo.click_count)
        .bind(&vo.cpc)
        .bind(&vo.thousand_show_cost)
        .bind(&vo.cost)
        .bind(&vo.download_count)
        .bind(&vo.download_cost)
        .bind(&vo.install_count)
        .bind(&vo.install_cost)
        .bind(&vo.active_count_normalized)
        .bind(&vo.active_cost_normalized)
        .bind(&vo.register_count)
        .bind(&vo.register_cost)
        .bind(&vo.retain_count_normalized)
        .bind(&vo.retain_cost_normalized)
        .bind(&vo.three_day_retain_count)
        .bind(&vo.three_day_retain_cost)
        .bind(&vo.subscribe_count)
        .bind(&vo.subscribe_cost)
        .bind(&vo.seven_day_retain_count)
        .bind(&vo.seven_day_retain_cost)
        .bind(&vo.publisher_real_price_one_day)
        .bind(&vo.ad_income_one_day_ltv_hms)
        .bind(&vo.ad_income_two_day_ltv_hms)
        .bind(&vo.ad_income_three_day_ltv_hms)
        .bind(&vo.ad_income_seven_day_ltv_hms)
        .bind(&vo.ad_income_fifteen_day_ltv_hms)
        .bind(&vo.ad_income_thirty_day_ltv_hms)
        .bind(&vo.ad_income_one_day_roi)
        .bind(&vo.ad_income_two_day_roi)
        .bind(&vo.ad_income_three_day_roi)
        .bind(&vo.ad_income_seven_day_roi)
        .bind(&vo.ad_income_fifteen_day_roi)
        .bind(&vo.ad_income_thirty_day_roi)
        .bind(&vo.attribution_income_iaa)
        .bind(&vo.attribution_income_iap_normalized)
        .bind(&vo.ad_position_id)
        .bind(&vo.country);
    }
    
    let cmd = query.sql();
    let rs = query.execute(pool).await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("azadmin.reports err {}", e);
            println!("{}", cmd);
        }
    }
}

pub async fn save_daily_release_report_group_by_advertiser(pool: &Pool<MySql>, data_list: &Vec<AdsDailyReleaseReportAdv>, record_date: &str) {
    if data_list.is_empty() {
        return;
    }
    let mut sql = "INSERT INTO azadmin.ads_advertiser_daily_release_reports
    (advertiser_id, package_name, cost, active, iaa, stat_datetime, record_datetime, country)
    VALUES ".to_string();

    let mut placeholder: Vec<&str> = vec![];
    for _ in data_list {
        placeholder.push("(?, ?, ?, ?, ?, ?, ?, ?)");
    }

    sql += placeholder.join(",").as_str();
    sql += "ON DUPLICATE KEY UPDATE cost=VALUES(cost),active=VALUES(active),iaa=VALUES(iaa)";

    let mut query = sqlx::query(sql.as_str());


    for vo in data_list {
        query = query.bind(&vo.advertiser_id)
        .bind(&vo.package_name)
        .bind(&vo.cost)
        .bind(&vo.active)
        .bind(&vo.iaa)
        .bind(&vo.stat_datetime)
        .bind(&record_date)
        .bind(&vo.country)
        ;
    }
    
    // let cmd = query.sql();
    let rs = query.execute(pool).await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("save_daily_release_report_group_by_advertiser {}", e);
            // println!("{}", cmd);
        }
    }
}

pub async fn save_admin(pool: &Pool<MySql>, param: &Admin) -> i32 {
    let rs = if let Some(id) = param.id {
        sqlx::query("UPDATE admin SET username=?, `password`=?, `name`=?, role=? WHERE id=?")
            .bind(&param.username)
            .bind(&param.password)
            .bind(&param.name)
            .bind(&param.role)
            .bind(&param.id)
            .execute(pool).await
    } else {
        sqlx::query("INSERT INTO admin (username, `password`, `name`, role) VALUES (?,?,?,?)")
            .bind(&param.username)
            .bind(&param.password)
            .bind(&param.name)
            .bind(&param.role)
            .execute(pool).await
    };
    
    match rs {
        Ok(v) => {
            0
        },
        Err(e) => {
            print!("save_admin: {}", e);
            1
        }
    }
}

pub async fn get_admin_company_id(pool: &Pool<MySql>, uid: i32) -> i32 {
    let rs = sqlx::query("SELECT company_id FROM admin WHERE id=?")
        .bind(uid)
        .fetch_one(pool).await;
    match rs {
        Ok(v) => {
            let v = v.get(0);
            v
        },
        Err(e) => {
            print!("get_admin_company_id: {}", e);
            0
        }
    }
}

pub async fn get_admin(pool: &Pool<MySql>, uid: i32) -> Option<Vec<Admin>> {
    let company_id = get_admin_company_id(pool, uid).await;
    let rs = sqlx::query_as::<_, Admin>("SELECT * FROM admin WHERE company_id=?")
        .bind(company_id)
        .fetch_all(pool).await;
    match rs {
        Ok(v) => Some(v),
        Err(e) => {
            print!("get_admin: {}", e);
            None
        }
    }
}

pub async fn get_admin_advertisers(pool: &Pool<MySql>) -> Option<Vec<AdminAdvertiser>> {
    let rs = sqlx::query_as::<_, AdminAdvertiser>("SELECT a.*, b.remark FROM admin_advertisers a LEFT JOIN advertisers b ON a.advertiser_id=b.advertiser_id")
            .fetch_all(pool).await;
    match rs {
        Ok(v) => Some(v),
        Err(e) => {
            print!("get_admin: {}", e);
            None
        }
    }
}

pub async fn save_admin_advertisers(pool: &Pool<MySql>, param: &ReqSaveAdminAdvertiser) -> i32 {
    let advertisers = param.advertisers.join(",");
    sqlx::query(format!("DELETE FROM admin_advertisers WHERE uid=? OR FIND_IN_SET(advertiser_id, '{}')", advertisers).as_str())
            .bind(&param.uid)
            .execute(pool).await;
    
    
    for advertiser_id in &param.advertisers {
        sqlx::query("INSERT INTO admin_advertisers (uid, advertiser_id) VALUES (?,?)")
            .bind(&param.uid)
            .bind(advertiser_id)
            .execute(pool).await;
    }
    0
}

pub async fn change_password(pool: &Pool<MySql>, uid: i32, password: &String) -> i32 {
    let rs = sqlx::query("UPDATE admin SET `password`=?, is_set_password=1 WHERE id=?")
            .bind(password)
            .bind(uid)
            .execute(pool).await;
    if rs.is_ok() {
        0
    } else {
        1
    }
}

pub async fn get_company_url(pool: &Pool<MySql>, company_id: i32) -> Option<String> {
    let rs = sqlx::query("SELECT url FROM admin WHERE company_id=?")
        .bind(company_id)
        .fetch_one(pool).await;
    match rs {
        Ok(v) => {
            let url: String = v.get(0);
            Some(url)
        },
        Err(e) => {
            print!("get_company_url: {}", e);
            None
        }
    }
}

pub async fn get_advertiser_tokens(pool: &Pool<MySql>) -> Vec<ReleaseToken> {
    let rs = sqlx::query_as::<_, ReleaseToken>("SELECT * FROM advertisers WHERE NOT ISNULL(access_token)")
        .fetch_all(pool)
        .await;
    match rs {
        Ok(v) => v,
        Err(e) => {
            println!("get_advertiser_tokens {}", e);
            vec![]
        }
    }
}

async fn get_assets_aid(pool: &Pool<MySql>, sha256: &String) -> i32 {
    let rs = sqlx::query("SELECT id FROM assets WHERE file_hash_sha256=?")
        .bind(sha256)
        .fetch_one(pool)
        .await;
    match rs {
        Ok(v) => {
            let id: i32 = v.get(0);
            id
        },
        Err(e) => {
            0
        }
    }
}


pub async fn save_assets(pool: &Pool<MySql>, advertiser_id: &str, inv: &ResQueryAssets) {
    let mut aid = get_assets_aid(pool, &inv.file_hash_sha256).await;
    if aid == 0 {
        let rs = sqlx::query("INSERT INTO azadmin.assets
        (assets_name, file_hash_sha256, file_url, asset_type, width, height, video_play_duration, file_size, file_format, create_time)
        VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP) ON DUPLICATE KEY UPDATE asset_type=VALUES(asset_type),width=VALUES(width),height=VALUES(height),video_play_duration=VALUES(video_play_duration),file_size=VALUES(file_size),file_format=VALUES(file_format);
        ")
        .bind(&inv.asset_name)
        .bind(&inv.file_hash_sha256)
        .bind(&inv.file_url)
        .bind(&inv.asset_type)
        .bind(&inv.width)
        .bind(&inv.height)
        .bind(&inv.video_play_duration)
        .bind(&inv.file_size)
        .bind(&inv.file_format)
        .execute(pool)
        .await;
        aid = get_assets_aid(pool, &inv.file_hash_sha256).await;
    }

    sqlx::query("INSERT INTO assets_advertiser (assets_id, aid, advertiser_id) SELECT ?,?,? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM assets_advertiser WHERE assets_id=?)")
        .bind(inv.asset_id)
        .bind(aid)
        .bind(advertiser_id)
        .bind(inv.asset_id)
        .execute(pool).await;
}

pub async fn save_assets_advertiser(pool: &Pool<MySql>, asset_id: &String, aid: i32, advertiser_id: &String) {
    sqlx::query("INSERT INTO assets_advertiser (assets_id, aid, advertiser_id) SELECT ?,?,? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM assets_advertiser WHERE assets_id=?)")
        .bind(asset_id)
        .bind(aid)
        .bind(advertiser_id)
        .execute(pool).await;
}

pub async fn update_position_detail(pool: &Pool<MySql>, creative_size_id: &String, txt: &str) {
    let rs = sqlx::query("UPDATE positions SET detail=? WHERE creative_size_id=?")
        .bind(txt)
        .bind(creative_size_id)
        .execute(pool).await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("update_position_detail {}", e)
        }
    }
}

pub async fn get_position_detail(pool: &Pool<MySql>, creative_size_id: &String) -> Option<String> {
    let rs = sqlx::query("SELECT detail FROM positions WHERE creative_size_id=?")
        .bind(creative_size_id)
        .fetch_one(pool)
        .await;
    match rs {
        Ok(v) => {
            let id: Option<String> = v.get(0);
            id
        },
        Err(e) => {
            None
        }
    }
}

pub async fn get_assets_url(pool: &Pool<MySql>, aid: i32) -> Option<String> {
    let rs = sqlx::query("SELECT IFNULL(file_url, local_path) FROM assets WHERE id=?")
        .bind(aid)
        .fetch_one(pool)
        .await;
    match rs {
        Ok(v) => {
            let id: Option<String> = v.get(0);
            id
        },
        Err(e) => {
            None
        }
    }
}

pub async fn query_assets(pool: &Pool<MySql>, req: &FormQueryAssets) -> Option<Vec<Assets>> {
    let rs = sqlx::query_as::<_, Assets>("SELECT * FROM assets WHERE asset_type = ? AND width = ? AND height = ? AND NOT ISNULL(local_path)")
        .bind(&req.asset_type)
        .bind(req.width)
        .bind(req.height)
        .fetch_all(pool)
        .await;
    match rs {
        Ok(v) => {
            Some(v)
        },
        Err(e) => {
            None
        }
    }
}

pub async fn get_uncollection_tasks(pool: &Pool<MySql>) -> Option<Vec<CollectionTask>> {
    let today = Local::now().format("%Y-%m-%d").to_string();

    let rs = sqlx::query_as::<_, CollectionTask>("SELECT a.* FROM collection_tasks a LEFT JOIN collection_tasks_records b ON a.id=b.task_id AND b.`date` =? WHERE ISNULL(b.task_id) AND a.enabled = 1")
        .bind(today)
        .fetch_all(pool)
        .await;
    match rs {
        Ok(v) => {
            Some(v)
        },
        Err(e) => {
            println!("get_uncollection_tasks: {}", e);
            None
        }
    }
}

pub async fn get_today_campaign_stat(pool: &Pool<MySql>) -> Option<Vec<CampaignStat>> {
    let today = Local::now().format("%Y-%m-%d").to_string();

    let rs = sqlx::query_as::<_, CampaignStat>("SELECT SUM(a.attribution_income_iaa) as iaa, sum(a.cost) as cost, a.campaign_id, a.advertiser_id  from reports a WHERE a.stat_datetime =? and a.cost>0.5 group by a.campaign_id, a.advertiser_id")
        .bind(today)
        .fetch_all(pool)
        .await;
    match rs {
        Ok(v) => {
            Some(v)
        },
        Err(e) => {
            println!("get_today_campaign_stat: {}", e);
            None
        }
    }
}

pub async fn done_collection_task(pool: &Pool<MySql>, id: i32) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    
    let rs = sqlx::query("INSERT INTO collection_tasks_records (task_id, `date`) SELECT ?,? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM collection_tasks_records WHERE task_id=? AND `date`=?)")
        .bind(id)
        .bind(&today)
        .bind(id)
        .bind(&today)
        .execute(pool)
        .await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("done_collection_task: {}", e);
        }
    }
}

pub async fn update_campaign_status(pool: &Pool<MySql>, campaign_id: &str, status: i32) {
    let rs = sqlx::query("INSERT INTO campaign_status (campaign_id, `status`) VALUES (?,?) ON DUPLICATE KEY UPDATE `status`=VALUES(`status`)")
        .bind(campaign_id)
        .bind(status)
        .execute(pool)
        .await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("update_campaign_status: {}", e);
        }
    }
}

pub async fn add_collection_task_execute_records(pool: &Pool<MySql>, today: &String, task_id: i32, operation: i32, campaign_id: &str, cost: f64, iaa: f64) {
    let rs = sqlx::query("INSERT INTO collection_task_execute_records (`date`, `task_id`, operation, campaign_id, cost, iaa) VALUES (?,?,?,?,?,?)")
        .bind(today)
        .bind(task_id)
        .bind(operation)
        .bind(campaign_id)
        .bind(cost)
        .bind(iaa)
        .execute(pool)
        .await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("add_collection_task_execute_records: {}", e);
        }
    }
}

pub async fn get_collection_tasks(pool: &Pool<MySql>) -> Option<Vec<CollectionTask>> {
    let rs = sqlx::query_as::<_, CollectionTask>("SELECT * FROM collection_tasks")
        .fetch_all(pool)
        .await;
    match rs {
        Ok(v) => {
            Some(v)
        },
        Err(e) => {
            println!("get_collection_tasks: {}", e);
            None
        }
    }
}

pub async fn update_collection_tasks(pool: &Pool<MySql>, param: &FormUpdateCollectionStatus) -> i32 {
    let rs = sqlx::query("UPDATE collection_tasks SET enabled=? WHERE id=?")
        .bind(param.enabled)
        .bind(param.task_id)
        .execute(pool)
        .await;
    match rs {
        Ok(v) => {
            0
        },
        Err(e) => {
            println!("update_collection_tasks: {}", e);
            1
        }
    }
}

pub async fn get_collection_operations(pool: &Pool<MySql>, param: &FormCollectionId) -> Option<Vec<CollectionExecuteRecords>> {
    let rs = sqlx::query_as::<_, CollectionExecuteRecords>("SELECT *, DATE_FORMAT(create_time, '%Y-%m-%d %H:%i:%s') AS create_time FROM collection_task_execute_records WHERE task_id=? ORDER BY id DESC")
        .bind(param.task_id)
        .fetch_all(pool)
        .await;
    match rs {
        Ok(v) => {
            Some(v)
        },
        Err(e) => {
            println!("get_collection_operations: {}", e);
            None
        }
    }
}

pub async fn update_collection_advertisers(pool: &Pool<MySql>, param: &FormUpdateCollectionAdvertisers) -> i32 {
    let rs = sqlx::query("UPDATE collection_tasks SET advertisers=? WHERE id=?")
        .bind(&param.advertisers)
        .bind(param.task_id)
        .execute(pool)
        .await;
    match rs {
        Ok(v) => {
            0
        },
        Err(e) => {
            println!("update_collection_advertisers: {}", e);
            1
        }
    }
}

pub async fn get_app_product_id(pool: &Pool<MySql>, advertiser_id: &String, app: i32) -> Option<String> {
    let rs = sqlx::query("SELECT b.product_id FROM apps a LEFT JOIN products b ON a.app_id = b.app_id WHERE a.id=? AND b.advertiser_id=?")
        .bind(app)
        .bind(advertiser_id)
        .fetch_one(pool)
        .await;
    match rs {
        Ok(v) => {
            let id = v.get(0);
            id
        },
        Err(e) => {
            println!("get_app_product_id: {}", e);
            None
        }
    }
}


pub async fn get_app_id(pool: &Pool<MySql>, app: i32) -> Option<String> {
    let rs = sqlx::query("SELECT app_id FROM apps WHERE id=?")
        .bind(app)
        .fetch_one(pool)
        .await;
    match rs {
        Ok(v) => {
            let id = v.get(0);
            Some(id)
        },
        Err(e) => {
            println!("get_app_id: {}", e);
            None
        }
    }
}

pub async fn save_product_id(pool: &Pool<MySql>, app_id: &str, advertiser_id: &str, product_id: &String) {
    let rs = sqlx::query("INSERT INTO azadmin.products
    (app_id, advertiser_id, product_id)
    VALUES(?, ?, ?) ON DUPLICATE KEY UPDATE product_id=VALUES(product_id);    
    ")
        .bind(app_id)
        .bind(advertiser_id)
        .bind(product_id)
        .execute(pool)
        .await;
    match rs {
        Ok(v) => {},
        Err(e) => {
            println!("save_product_id: {}", e);
        }
    }
}

pub async fn get_asset_id(pool: &Pool<MySql>, aid: i32, advertiser_id: &String) -> Option<i64> {
    let rs = sqlx::query("SELECT assets_id FROM assets_advertiser WHERE aid=? AND advertiser_id=?
    ")
        .bind(aid)
        .bind(advertiser_id)
        .fetch_one(pool)
        .await;
    match rs {
        Ok(v) => {
            let rs = v.get(0);
            rs
        },
        Err(e) => {
            println!("get_asset_id: {}", e);
            None
        }
    }
}

pub async fn get_assets_name(pool: &Pool<MySql>, aid: i32) -> String {
    let rs = sqlx::query("SELECT assets_name FROM assets WHERE id=?")
        .bind(aid)
        .fetch_one(pool)
        .await;
    match rs {
        Ok(v) => {
            let rs: Option<&str> = v.get(0);
            rs.unwrap().to_string()
        },
        Err(e) => {
            println!("get_asset_id: {}", e);
            "".to_string()
        }
    }
}
