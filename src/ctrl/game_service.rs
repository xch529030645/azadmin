use std::{time::{SystemTime, UNIX_EPOCH}, collections::{HashMap, HashSet}, ops::Sub};
use chrono::{Local, DateTime, Days};
use sqlx::{Pool, MySql, Row};

use crate::{lib::{server_api, req::{AuthorizationCode}, response::*, umeng_api}, model::*, auth, user_data::UserData};

use super::game_repository;
// use async_recursion::async_recursion;

const CLIENT_ID: &str = "108128867";
const CLIENT_SECRET: &str = "9fae7c0bf88cd122d3fc87d1709fba4326d1f157a5d82a6aa1b4cf7123c09d54";
const REDIRECT_URL: &str = "https://dat.311419.cn/azadmin/auth";

#[derive(Clone)]
pub struct GameService {

}

impl GameService {
    pub fn create() -> GameService {
        GameService {}
    }

    pub async fn authcallback(&self, pool: &Pool<MySql>, auth: &AuthorizationCode) {
        // println!("authcallback: {}", &auth.authorization_code);
        let rs = server_api::get_access_token(&auth.authorization_code, CLIENT_ID, CLIENT_SECRET, REDIRECT_URL).await;
        let advertiser_id = &auth.state;
        if let Some(res_access_token) = rs {
            self.save_market_access_token(pool, &res_access_token, &advertiser_id).await;
        } else {
            println!("no access token")
        }
    }

    pub async fn get_advertisers(&self, pool: &Pool<MySql>) -> Option<Vec<ReleaseToken>> {
        let rs = sqlx::query_as::<_, ReleaseToken>("SELECT * FROM advertisers")
        .fetch_all(pool)
        .await;
        match rs {
            Ok(v) => Some(v),
            Err(e) => {
                println!("get_advertisers {}", e);
                None
            }
        }
    }

    pub async fn add_advertiser(&self, pool: &Pool<MySql>, param: &ReqAddAdv) -> i32 {
        game_repository::add_advertiser(pool, &param.advertiser_id, CLIENT_ID, CLIENT_SECRET, REDIRECT_URL).await
    }

    pub async fn update_advertiser_remark(&self, pool: &Pool<MySql>, param: &ReqAdvRemark) -> i32 {
        game_repository::update_advertiser_remark(pool, &param).await
    }

    pub async fn save_admin(&self, pool: &Pool<MySql>, param: &Admin) -> i32 {
        game_repository::save_admin(pool, param).await
    }

    pub async fn get_admin(&self, pool: &Pool<MySql>) -> Option<Vec<Admin>> {
        game_repository::get_admin(pool).await
    }

    pub async fn get_admin_advertisers(&self, pool: &Pool<MySql>) -> Option<Vec<AdminAdvertiser>> {
        game_repository::get_admin_advertisers(pool).await
    }

    pub async fn save_admin_advertisers(&self, pool: &Pool<MySql>, param: &ReqSaveAdminAdvertiser) -> i32 {
        game_repository::save_admin_advertisers(pool, param).await
    }
    
    pub async fn get_app_gallery(&self, pool: &Pool<MySql>) -> Option<Vec<AppGallery>> {
        let rs = sqlx::query_as::<_, AppGallery>("SELECT * FROM ads_account")
        .fetch_all(pool)
        .await;
        match rs {
            Ok(v) => Some(v),
            Err(e) => {
                println!("get_app_gallery {}", e);
                None
            }
        }
    }

    pub async fn add_app_gallery(&self, pool: &Pool<MySql>, param: &ReqAddGallery) -> i32 {
        game_repository::add_app_gallery(pool, &param.client_id, &param.client_secret, &param.connect_client_id, &param.connect_client_secret, &param.remark).await
    }

    fn get_report_query_conds(&self, params: &ReqQueryReports) -> Vec<String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let yesterday = Local::now().checked_sub_days(Days::new(1)).unwrap().format("%Y-%m-%d").to_string();
        let mut conds: Vec<String> = vec![];
        conds.push(format!("(a.record_datetime='{}' OR a.record_datetime='{}')", today, yesterday));
        if let Some(package_name) = &params.package_name {
            conds.push(format!("FIND_IN_SET(a.package_name,'{}')", package_name));
        }
        if let Some(start_date) = &params.start_date {
            conds.push(format!("a.stat_datetime>='{}'", start_date));
        }
        if let Some(end_date) = &params.end_date {
            conds.push(format!("a.stat_datetime<='{}'", end_date));
        }
        if let Some(country) = &params.country {
            if country.eq("ALL") {
                if params.group_by_country {
                    conds.push("a.country!='ALL'".to_string());
                } else {
                    conds.push("a.country='ALL'".to_string());
                }
            } else {
                conds.push(format!("FIND_IN_SET(a.country, '{}')", country));
            }
        } else {
            if params.group_by_country {
                conds.push("a.country!='ALL'".to_string());
            } else {
                conds.push("a.country='ALL'".to_string());
            }
        }
        if let Some(ads_accounts) = &params.ads_accounts {
            if !ads_accounts.is_empty() {
                conds.push(format!("FIND_IN_SET(b.client_id, '{}')", ads_accounts));
            }
        }
        if let Some(advertisers) = &params.advertisers {
            if !advertisers.is_empty() {
                conds.push(format!("FIND_IN_SET(a.advertiser_id, '{}')", advertisers));
            }
        }
        conds
    }

    async fn query_release_reports(&self, pool: &Pool<MySql>, params: &ReqQueryReports, conds: &Vec<String>) -> Option<Vec<ResAdsReports>>{
        let table = if let Some(advertisers) = &params.advertisers {
            if advertisers.is_empty() {
                "ads_daily_release_reports"
            } else {
                "ads_advertiser_daily_release_reports"
            }
        } else {
            "ads_daily_release_reports"
        };
        let left_join_cond = if table.eq("ads_daily_release_reports") {
            ""
        } else {
            "AND a.advertiser_id=d.advertiser_id"
        };
        let app_name_format = if params.group_by_country {
            "CONCAT(b.app_name,'-',a.country) AS app_name"
        } else {
            "b.app_name"
        };
        let mut sql = format!("SELECT * FROM (SELECT a.package_name, SUM(a.cost) AS cost, CAST(SUM(a.active) AS SIGNED) as active, SUM(a.iaa) AS iaa, {}, SUM(c.earnings) AS earnings, SUM(d.iaa) as first_day_iaa, CAST(AVG(f.duration) AS SIGNED) AS duration, AVG(f.r1) AS r1, g.remark
        FROM {} a 
        LEFT JOIN apps b ON a.package_name = b.package_name 
        LEFT JOIN ads_daily_earnings_reports c ON b.app_id = c.app_id AND c.stat_datetime=a.stat_datetime 
        LEFT JOIN {} d ON a.package_name = d.package_name AND a.stat_datetime = d.stat_datetime and d.record_datetime = a.stat_datetime and a.country=d.country {}
        LEFT JOIN um_apps e ON e.package_name = a.package_name 
        LEFT JOIN um_retention f ON e.appkey = f.appkey AND f.date=a.stat_datetime 
        LEFT JOIN ads_account g ON b.client_id=g.client_id ", app_name_format, table, table, left_join_cond);
        

        if !conds.is_empty() {
            sql += format!(" WHERE {}", conds.join(" AND ")).as_str();
        }
        // if let Some(_) = &params.country {
        //     sql += " GROUP BY a.stat_datetime, a.package_name, b.app_name, c.earnings"
        // } else {
        //     sql += " GROUP BY a.stat_datetime, a.package_name, b.app_name, c.earnings"
        // }
        let mut group_by = [
            "a.package_name", "b.app_name", "g.remark"
        ].to_vec();
        if params.group_by_country {
            group_by.push("a.country");
        }
        sql += " GROUP BY ";
        sql += group_by.join(",").as_str();

        // cost, active, iaa, earnings
        let order_prop = match &params.order_prop {
            Some(order_prop) => {
                match order_prop.as_str() {
                    "cost" => "t.cost",
                    "active" => "t.active",
                    "iaa" => "t.iaa",
                    "earnings" => "t.earnings",
                    _ => "t.cost"
                }
            },
            None => {
                "t.cost"
            }
        };

        let order = match &params.order {
            Some(order) => {
                match order.as_str() {
                    "descending" => "DESC",
                    "ascending" => "ASC",
                    _ => "DESC"
                }
            },
            None => {
                "DESC"
            }
        };

        sql += format!(") t ORDER BY {} {} LIMIT {}, {}", order_prop, order, params.page * params.len, params.len).as_str();
        // println!("{}", &sql);

        let rs = sqlx::query_as::<_, ResAdsReports>(sql.as_str())
        .fetch_all(pool)
        .await;
        match rs {
            Ok(v) => Some(v),
            Err(e) => {
                println!("get_reports err {}", e);
                None
            }
        }
    }

    pub async fn get_reports(&self, pool: &Pool<MySql>, params: &ReqQueryReports) -> Option<ResGetReports> {
        let conds = self.get_report_query_conds(params);
        let list = self.query_release_reports(pool, params, &conds).await;

        let table = if let Some(advertisers) = &params.advertisers {
            if advertisers.is_empty() {
                "ads_daily_release_reports"
            } else {
                "ads_advertiser_daily_release_reports"
            }
        } else {
            "ads_daily_release_reports"
        };
        
        let mut sql = format!("SELECT COUNT(*) AS `count` FROM {} a", table);
        if !conds.is_empty() {
            sql += format!(" WHERE {}", conds.join(" AND ")).as_str();
        }
        let count_rs = sqlx::query_as::<_, ResAdsReportsCount>(sql.as_str())
        .fetch_one(pool)
        .await;
        let count = match count_rs {
            Ok(v) => v.count,
            Err(e) => {
                println!("get_reports count {}", e);
                0
            }
        };

        if let Some(list) = list {
            Some(ResGetReports {
                list, total_number: count
            })
        } else {
            None
        }
    }

    pub async fn get_overview(&self, pool: &Pool<MySql>, params: &ReqQueryOverview) -> Option<Vec<ResOverview>> {
        let rs = sqlx::query_as::<_, ResOverview>("SELECT a.cost, DATE_FORMAT(a.stat_datetime, '%Y-%m-%d') as stat_datetime, b.earnings FROM (
            SELECT SUM(cost) as cost, stat_datetime FROM ads_daily_release_reports 
            WHERE record_datetime = stat_datetime and stat_datetime BETWEEN ? AND ? AND country ='ALL' group by stat_datetime
            ) a 
            LEFT JOIN 
            (
            SELECT SUM(earnings) AS earnings, stat_datetime FROM ads_daily_earnings_reports WHERE stat_datetime BETWEEN ? AND ? GROUP BY stat_datetime
            ) b
            ON a.stat_datetime = b.stat_datetime;")
        .bind(&params.start_date)
        .bind(&params.end_date)
        .bind(&params.start_date)
        .bind(&params.end_date)
        .fetch_all(pool)
        .await;
        match rs {
            Ok(v) => Some(v),
            Err(e) => {
                println!("get_overview err {}", e);
                None
            }
        }
    }

    pub async fn get_sum_reports(&self, pool: &Pool<MySql>, params: &ReqQueryReports) -> Option<ResSumReports> {
        // let today = Local::now().format("%Y-%m-%d").to_string();
        // let yesterday = Local::now().checked_sub_days(Days::new(1)).unwrap().format("%Y-%m-%d").to_string();

        let conds = self.get_report_query_conds(params);

        let table = if let Some(advertisers) = &params.advertisers {
            if advertisers.is_empty() {
                "ads_daily_release_reports"
            } else {
                "ads_advertiser_daily_release_reports"
            }
        } else {
            "ads_daily_release_reports"
        };
        let left_join_cond = if table.eq("ads_daily_release_reports") {
            ""
        } else {
            "AND a.advertiser_id=d.advertiser_id"
        };

        
        let mut sql = format!("SELECT SUM(a.cost) as cost, CAST(SUM(a.active) as SIGNED) as active, SUM(a.iaa) as iaa, SUM(c.earnings) as earnings, SUM(d.iaa) as first_day_iaa 
        FROM {} a 
        LEFT JOIN apps b ON a.package_name = b.package_name 
        LEFT JOIN ads_daily_earnings_reports c ON b.app_id = c.app_id AND c.stat_datetime=a.stat_datetime 
        LEFT JOIN {} d ON a.package_name = d.package_name AND a.stat_datetime = d.stat_datetime and d.record_datetime = a.stat_datetime and a.country=d.country {} ",
        table, table, left_join_cond);
        // let mut conds: Vec<String> = vec![];
        // conds.push(format!("(a.record_datetime='{}' OR a.record_datetime='{}')", today, yesterday));
        // if let Some(package_name) = &params.package_name {
        //     conds.push(format!("FIND_IN_SET(a.package_name,'{}')", package_name));
        // }
        // if let Some(start_date) = &params.start_date {
        //     conds.push(format!("a.stat_datetime>='{}'", start_date));
        // }
        // if let Some(end_date) = &params.end_date {
        //     conds.push(format!("a.stat_datetime<='{}'", end_date));
        // }
        // if let Some(country) = &params.country {
        //     conds.push(format!("FIND_IN_SET(a.country, '{}')", country));
        // }

        if !conds.is_empty() {
            sql += format!(" WHERE {}", conds.join(" AND ")).as_str();
        }

        let rs = sqlx::query_as::<_, ResSumReports>(sql.as_str())
        .fetch_one(pool)
        .await;
        match rs {
            Ok(v) => Some(v),
            Err(e) => {
                println!("get_sum_reports err {}", e);
                None
            }
        }
    }

    pub async fn generate_reports_csv(&self, pool: &Pool<MySql>, params: &ReqQueryReports) -> String {
        let conds = self.get_report_query_conds(params);

        // let today = Local::now().format("%Y-%m-%d").to_string();
        // let yesterday = Local::now().checked_sub_days(Days::new(1)).unwrap().format("%Y-%m-%d").to_string();
        
        // let mut sql = "SELECT a.package_name, a.cost, a.active, a.iaa, DATE_FORMAT(a.stat_datetime , '%Y-%m-%d') as stat_datetime, b.app_name, c.earnings, d.iaa as first_day_iaa, f.duration, f.r1, g.remark
        // FROM ads_daily_release_reports a 
        // LEFT JOIN apps b ON a.package_name = b.package_name 
        // LEFT JOIN ads_daily_earnings_reports c ON b.app_id = c.app_id AND c.stat_datetime=a.stat_datetime 
        // LEFT JOIN ads_daily_release_reports d ON a.package_name = d.package_name AND a.stat_datetime = d.stat_datetime and d.record_datetime = a.stat_datetime and a.country=d.country 
        // LEFT JOIN um_apps e ON e.package_name = a.package_name 
        // LEFT JOIN um_retention f ON e.appkey = f.appkey AND f.date=a.stat_datetime 
        // LEFT JOIN ads_account g ON b.client_id=g.client_id ".to_string();
        // let mut conds: Vec<String> = vec![];
        // conds.push(format!("(a.record_datetime='{}' OR a.record_datetime='{}')", today, yesterday));
        // if let Some(package_name) = &params.package_name {
        //     conds.push(format!("FIND_IN_SET(a.package_name,'{}')", package_name));
        // }
        // if let Some(start_date) = &params.start_date {
        //     conds.push(format!("a.stat_datetime>='{}'", start_date));
        // }
        // if let Some(end_date) = &params.end_date {
        //     conds.push(format!("a.stat_datetime<='{}'", end_date));
        // }
        // if let Some(country) = &params.country {
        //     conds.push(format!("FIND_IN_SET(a.country, '{}')", country));
        // }

        // if !conds.is_empty() {
        //     sql += format!(" WHERE {}", conds.join(" AND ")).as_str();
        // }

        // // cost, active, iaa, earnings
        // let order_prop = match &params.order_prop {
        //     Some(order_prop) => {
        //         match order_prop.as_str() {
        //             "cost" => "a.cost",
        //             "active" => "a.active",
        //             "iaa" => "a.iaa",
        //             "earnings" => "c.earnings",
        //             _ => "a.stat_datetime"
        //         }
        //     },
        //     None => {
        //         "a.stat_datetime"
        //     }
        // };

        // let order = match &params.order {
        //     Some(order) => {
        //         match order.as_str() {
        //             "descending" => "DESC",
        //             "ascending" => "ASC",
        //             _ => "DESC"
        //         }
        //     },
        //     None => {
        //         "DESC"
        //     }
        // };

        // sql += format!(" ORDER BY {} {}", order_prop, order).as_str();
        // // println!("{}", &sql);

        // let rs = sqlx::query_as::<_, ResAdsReports>(sql.as_str())
        // .fetch_all(pool)
        // .await;
        // let list = match rs {
        //     Ok(v) => Some(v),
        //     Err(e) => {
        //         println!("get_reports err {}", e);
        //         None
        //     }
        // };
        let rs = self.query_release_reports(pool, params, &conds).await;

        let mut s = String::from("");
        s.push_str("App Name,Cost,Active,CPI,IAA,Revenue,ROAS,ROAS of today,Duration,Retain,Advertiser\n");
        if let Some(list) = rs {
            for row in list {
                let cpi = if row.active > 0 {
                    row.cost / (row.active as f64)
                } else {
                    0_f64
                };
                let roas = if row.cost > 0_f64 {
                    row.iaa / row.cost
                } else {
                    0_f64
                };
                let first_day_roas = if row.cost > 0_f64 {
                    row.first_day_iaa.unwrap_or(0_f64) / row.cost
                } else {
                    0_f64
                };
                let retain = if let Some(retain) = &row.r1 {
                    retain.to_string()
                } else {
                    "".to_string()
                };
                let duration = if let Some(duration) = &row.duration {
                    duration.to_string()
                } else {
                    "".to_string()
                };
                s.push_str(format!("{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?}\n", 
                    &row.app_name.unwrap_or("Unknown".to_string()),
                    // &row.stat_datetime,
                    &row.cost,
                    &row.active,
                    cpi,
                    &row.iaa,
                    &row.earnings.unwrap_or(0_f64),
                    roas,
                    first_day_roas,
                    duration,
                    retain,
                    &row.remark.unwrap_or("".to_string())
                ).as_str());
            }
        }
        s
        
    }
    
    pub async fn get_apps(&self, pool: &Pool<MySql>) -> Option<Vec<App>> {
        let rs = sqlx::query_as::<_, App>("SELECT a.*, b.appkey FROM apps a LEFT JOIN um_apps b ON a.package_name=b.package_name")
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

    pub async fn get_ads_accounts(&self, pool: &Pool<MySql>) -> Option<Vec<AdsAccount>> {
        game_repository::get_ads_accounts(pool).await
    }

    pub async fn bind_app(&self, pool: &Pool<MySql>, param: &ReqBindApp) -> i32 {
        game_repository::bind_app(pool, param).await
    }

    pub async fn set_umkey(&self, pool: &Pool<MySql>, param: &ReqBindUmKey) -> i32 {
        game_repository::set_umkey(pool, param).await
    }

    pub async fn get_app_roas(&self, pool: &Pool<MySql>, param: &ReqRoas) -> Option<Vec<AdsRoas>> {
        game_repository::get_app_roas(pool, param).await
    }

    pub async fn login_admin(&self, pool: &Pool<MySql>, req: &ReqLogin) -> Option<ResLogin> {
        let rs = sqlx::query_as::<_, AdminInfo>("SELECT a.id, a.`password`, b.prev FROM admin a LEFT JOIN `privileges` b ON a.role=b.role WHERE a.username=?")
        .bind(&req.username)
        .fetch_one(pool)
        .await;

        match rs {
            Ok(v) => {
                if v.password.eq(&req.password) {
                    let jwt = auth::create_jwt(&v.id);
                    Some(ResLogin {token: jwt, privileges: v.prev})
                } else {
                    None
                }
            },
            Err(_) => None
        }
    }

    pub async fn get_privileges(&self, pool: &Pool<MySql>, user_data: &UserData) -> Option<String> {
        let rs = sqlx::query("SELECT b.prev FROM admin a LEFT JOIN `privileges` b ON a.role=b.role WHERE a.id=?")
        .bind(&user_data.id)
        .fetch_one(pool)
        .await;

        match rs {
            Ok(v) => {
                let prev: String = v.get(0);
                Some(prev)
            },
            Err(_) => None
        }
    }

    pub async fn check_access_token(&self, pool: &Pool<MySql>) {
        self.check_market_access_token(pool).await;
        self.check_ads_access_token(pool).await;
        self.check_connect_access_token(pool).await;
    }

    async fn check_market_access_token(&self, pool: &Pool<MySql>) {
        // println!();
        // println!("==check_market_access_token start==");
        let rs = sqlx::query_as::<_, ReleaseClientToken>("SELECT * from advertisers WHERE (ISNULL(expires_in) OR expires_in < UNIX_TIMESTAMP()*1000+3600) AND NOT ISNULL(access_token)")
        .fetch_all(pool)
        .await;
        match rs {
            Ok(list) => {
                for ads_token in list {
                    let rs_token = server_api::refresh_token(&ads_token.refresh_token, &ads_token.client_id, &ads_token.secret).await;
                    if let Some(token) = rs_token {
                        self.save_market_access_token(pool, &token, &ads_token.advertiser_id).await;
                    } else {
                        println!("check_market_access_token no token");
                    }
                }
            },
            Err(e) => {
                println!("check_ads_access_token {}", e);
            }
        }
        // println!("==check_market_access_token end==");
        // println!();
    }

    async fn save_market_access_token(&self, pool: &Pool<MySql>, res_access_token: &ResAccessToken, advertiser_id: &String) {
        let expires_in_i64 = (res_access_token.expires_in * 1000) as i64;
            let expire_time = self.timestamp() + expires_in_i64 - 60000;
            let rs = sqlx::query("CALL p_save_advertiser(?,?,?,?)")
                .bind(advertiser_id)
                .bind(&res_access_token.access_token)
                .bind(&res_access_token.refresh_token)
                .bind(expire_time)
                .execute(pool).await;

            match rs {
                Ok(v) => {
                    println!("insert advertiser: {}", advertiser_id);
                },
                Err(e) => {
                    println!("p_save_advertiser err {}", e);
                }
            }
    }

    async fn check_ads_access_token(&self, pool: &Pool<MySql>) {
        let rs = game_repository::get_expired_ads_token(pool).await;
        match rs {
            Some(list) => {
                for ads_token in list {
                    let rs_token = server_api::get_ads_access_token(&ads_token).await;
                    if let Some(token) = rs_token {
                        self.save_ads_access_token(pool, &token, &ads_token).await;
                    }
                }
            },
            None => {}
        }
    }

    async fn check_connect_access_token(&self, pool: &Pool<MySql>) {
        let rs = game_repository::get_expired_connect_token(pool).await;
        match rs {
            Some(list) => {
                for ads_token in list {
                    let rs_token = server_api::get_connect_api_access_token(&ads_token).await;
                    if let Some(token) = rs_token {
                        self.save_connect_access_token(pool, &token, &ads_token).await;
                    }
                }
            },
            None => {}
        }
    }

    async fn save_ads_access_token(&self, pool: &Pool<MySql>, token: &ResAdsAccessToken, ads_token: &AdsToken) {
        println!("save_ads_access_token {}", token.access_token);
        let expire_time = token.expires_in as i64 * 1000_i64 + self.timestamp() - 60000_i64;
        let rs = sqlx::query("UPDATE ads_account SET access_token=?, expire_time=? WHERE client_id=?")
                .bind(&token.access_token)
                .bind(&expire_time)
                .bind(&ads_token.client_id)
                .execute(pool).await;
        match rs {
            Ok(_) => println!("save_ads_access_token for client {} succeed", &ads_token.client_id),
            Err(e) => println!("save_ads_access_token for client {} err: {}", &ads_token.client_id, e)
        }
    }

    async fn save_connect_access_token(&self, pool: &Pool<MySql>, token: &ResAdsAccessToken, ads_token: &ConnectToken) {
        println!("save_connect_access_token {}", token.access_token);
        let expire_time = token.expires_in as i64 * 1000_i64 + self.timestamp() - 60000_i64;
        let rs = sqlx::query("UPDATE ads_account SET connect_access_token=?, connect_expire_time=? WHERE client_id=?")
                .bind(&token.access_token)
                .bind(&expire_time)
                .bind(&ads_token.client_id)
                .execute(pool).await;
        match rs {
            Ok(_) => println!("save_connect_access_token for client {} succeed", &ads_token.client_id),
            Err(e) => println!("save_connect_access_token for client {} err: {}", &ads_token.client_id, e)
        }
    }

    pub async fn query_reports(&self, pool: &Pool<MySql>, start_date_local: &DateTime<Local>, end_date_local: &DateTime<Local>) {
        println!();
        println!("==query_reports start==");
        let rs = sqlx::query_as::<_, ReleaseToken>("SELECT * FROM advertisers WHERE NOT ISNULL(access_token)")
        .fetch_all(pool)
        .await;
        match rs {
            Ok(v) => {
                let mut days_range: Vec<Vec<String>> = vec![];
                let mut days: Vec<String> = vec![];
                let mut start_date = start_date_local.format("%Y-%m-%d").to_string();
                let end_date = end_date_local.format("%Y-%m-%d").to_string();
                if start_date.eq(&end_date) {
                    days.push(start_date.clone());
                    days_range.push(days);
                } else {
                    let mut from = start_date_local.clone();
                    let mut idx = 0;
                    while !start_date.eq(&end_date) {
                        if let Some(s) = from.checked_add_days(Days::new(1)) {
                            start_date = s.format("%Y-%m-%d").to_string();
                            println!("{}", &start_date);
                            days.push(start_date.clone());
                            from = s;
                            idx = idx + 1;
                            if idx > 6 {
                                idx = 0;
                                days_range.push(days);
                                days = vec![];
                            }
                        } else {
                            break;
                        }
                    }
                    if !days.is_empty() {
                        days_range.push(days);
                    }
                }
                let mut thread_headlers = vec![];
                for adv in &v {
                    if !&days_range.is_empty() {
                        let mut adv_token_copy = adv.clone();
                        let mysql = pool.clone();
                        let service = self.clone();
                        let days_range_copy = days_range.clone();
                        let handle = actix_rt::spawn(async move {
                            for days in days_range_copy {
                                
                                // let record_date = end_date.clone();
                                
                                // let service = GameService::create();
                                // let mut app_package_names: HashSet<String> = HashSet::new();
                                for date in days {
                                    println!("query {}, from: {}, to: {}", adv_token_copy.advertiser_id, &date, &date);
                                    let mut page_info: Option<PageInfo> = None;
                                    loop {
                                        let access_token = game_repository::get_marketing_access_token(&mysql, &adv_token_copy.advertiser_id).await;
                                        adv_token_copy.access_token = access_token;
                                        page_info = service.query_advertiser_reports(&mysql, &adv_token_copy, &date, &date, page_info, 1000).await;
                                        if let Some(pi) = page_info {
                                            if pi.page <= pi.total_page {
                                                page_info = Some(pi);
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                    // service.calc_release_daily_reports(&mysql, &date, &record_date, &mut app_package_names).await;
                                }

                                
                                // service.save_unknown_package_names(&mysql, &app_package_names).await;
                                
                            }

                        });
                        thread_headlers.push(handle);
                    }
                }
                for handle in thread_headlers {
                    if !handle.is_finished() {
                        handle.await;
                    }
                }
                println!("start calc_release_daily_reports");
                let mut app_package_names: HashSet<String> = HashSet::new();
                if !&days_range.is_empty() {
                    for days in &days_range {
                        for date in days {
                            self.calc_release_daily_reports(&pool, &date, &end_date, &mut app_package_names).await;
                            self.calc_advertiser_release_daily_reports(&pool, &date, &end_date).await;
                        }
                    }
                }
                self.save_unknown_package_names(&pool, &app_package_names).await;
            },
            Err(e) => {
                println!("query_reports err: {}", e);
            }
        }
        println!("==query_reports ends==");
        println!();
    }

    async fn save_unknown_package_names(&self, pool: &Pool<MySql>, app_package_names: &HashSet<String>) {
        for package_name in app_package_names {
            let is_set = game_repository::is_package_name_set(pool, package_name).await;
            if !is_set {
                game_repository::save_unknown_package_name(pool, package_name).await;
            }
        }
        
    }

    async fn calc_release_daily_reports(&self, pool: &Pool<MySql>, today: &String, record_date: &String, app_package_names: &mut HashSet<String>) {
        let list = game_repository::calc_ads_daily_release_reports_by_date(pool, &today).await;
        if let Some(list) = list {
            let now = self.timestamp();
            for vo in list {
                app_package_names.insert(vo.package_name.clone());
                if vo.cost == 0_f64 && vo.active == 0 && vo.iaa == 0_f64 {
                    continue;
                }
                game_repository::insert_or_update_daily_release_report(pool, &vo, record_date).await;
            }
            println!("calc_release_daily_reports use {}", self.timestamp() - now);
        }
    }

    async fn calc_advertiser_release_daily_reports(&self, pool: &Pool<MySql>, today: &String, record_date: &String) {
        let list = game_repository::calc_ads_daily_release_reports_group_by_advertiser_by_date(pool, &today).await;
        if let Some(list) = list {
            let now = self.timestamp();
            let mut data_list: Vec<AdsDailyReleaseReportAdv> = vec![];
            for vo in list {
                if vo.cost == 0_f64 && vo.active == 0 && vo.iaa == 0_f64 {
                    continue;
                }
                data_list.push(vo);
            }
            game_repository::save_daily_release_report_group_by_advertiser(pool, &data_list, record_date).await;
            println!("calc_advertiser_release_daily_reports use {}", self.timestamp() - now);
        }
    }

    // async fn query_package_app_id(&self, pool: &Pool<MySql>, client_id: &String, access_token: &String, app_package_names: &HashSet<String>) {
    //     for package_name in app_package_names {
    //         let rs = server_api::query_package_app_id(client_id, access_token, package_name).await;
    //         if let Some(rs) = rs {
    //             if rs.ret.code == 0 {
                    
    //             }
    //         }
    //     }
        
    // }
    

    fn is_zero_report(&self, vo: &ResReportVo) -> bool {
        return vo.show_count == 0 && vo.click_count == 0 && vo.cpc.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.thousand_show_cost.parse::<f32>().unwrap() == 0_f32 && vo.cost.parse::<f32>().unwrap_or(0_f32) == 0_f32
            && vo.download_count == 0 && vo.download_cost.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.install_count == 0 && vo.install_cost.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.active_count_normalized == 0 && vo.active_cost_normalized.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.register_count == 0 && vo.register_cost.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.retain_count_normalized == 0 && vo.retain_cost_normalized.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.three_day_retain_count == 0 && vo.three_day_retain_cost.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.subscribe_count == 0 && vo.subscribe_cost.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.seven_day_retain_count == 0 && vo.seven_day_retain_cost.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.publisher_real_price_one_day.parse::<f32>().unwrap_or(0_f32) == 0_f32 && vo.ad_income_one_day_ltv_hms.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.ad_income_two_day_ltv_hms.parse::<f32>().unwrap_or(0_f32) == 0_f32 && vo.ad_income_three_day_ltv_hms.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.ad_income_seven_day_ltv_hms.parse::<f32>().unwrap_or(0_f32) == 0_f32 && vo.ad_income_fifteen_day_ltv_hms.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.ad_income_thirty_day_ltv_hms.parse::<f32>().unwrap_or(0_f32) == 0_f32 && vo.ad_income_one_day_roi.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.ad_income_two_day_roi.parse::<f32>().unwrap_or(0_f32) == 0_f32 && vo.ad_income_three_day_roi.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.ad_income_seven_day_roi.parse::<f32>().unwrap_or(0_f32) == 0_f32 && vo.ad_income_fifteen_day_roi.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.ad_income_thirty_day_roi.parse::<f32>().unwrap_or(0_f32) == 0_f32 && vo.attribution_income_iaa.parse::<f32>().unwrap_or(0_f32) == 0_f32 
            && vo.attribution_income_iap_normalized.parse::<f32>().unwrap_or(0_f32) == 0_f32;
    }
    
    async fn query_advertiser_reports(&self, pool: &Pool<MySql>, advertiser: &ReleaseToken, start_date: &String, end_date: &String, page_info: Option<PageInfo>, page_size: i32) -> Option<PageInfo> {
        let page = match &page_info {
            Some(v) => v.page,
            None => 1
        };
        println!("query_advertiser_reports page {}", page);
        if let Some(access_token) = &advertiser.access_token {
            let rs = server_api::query_reports(&advertiser.advertiser_id, access_token, &start_date, &end_date, page, page_size).await;
            if let Some(reports) = rs {
                if reports.code == "0" {
                    // let mut sqls = Vec::<String>::new();
                    if let Some(data) = reports.data {
                        let list = data.list;
                        let mut data_list: Vec<Vec<ResReportVo>> = vec![];
                        let mut sub_list: Vec<ResReportVo> = vec![];
                        let mut idx = 0;
                        for item in list {
                            if !self.is_zero_report(&item) {
                                idx = idx + 1;
                                sub_list.push(item);
                                if idx >= 30 {
                                    data_list.push(sub_list);
                                    sub_list = vec![];
                                }
                            }
                        }
                        if !sub_list.is_empty() {
                            data_list.push(sub_list);
                        }

                        if !data_list.is_empty() {
                            let now = self.timestamp();
                            for list in data_list {
                                game_repository::save_marketing_reports(pool, &advertiser, &list).await;
                            }
                            println!("query_advertiser_reports use {}", self.timestamp() - now);
                        }
                        

                        /*
                        for vo in data_list {
                           
                            let stat_datetime = "".to_string() + &vo.stat_datetime[0..4] + "-" + &vo.stat_datetime[4..6] + "-" + &vo.stat_datetime[6..8];
                            let sql = "INSERT INTO azadmin.reports
                            (advertiser_id, adgroup_id, adgroup_name, campaign_id, campaign_name, package_name, stat_datetime, show_count, click_count, cpc, thousand_show_cost, cost, download_count, download_cost, install_count, install_cost, active_count, active_cost, register_count, register_cost, retain_count, retain_cost, three_day_retain_count, three_day_retain_cost, subscribe_count, subscribe_cost, seven_day_retain_count, seven_day_retain_cost, publisher_real_price_one_day, ad_income_one_day_ltv_hms, ad_income_two_day_ltv_hms, ad_income_three_day_ltv_hms, ad_income_seven_day_ltv_hms, ad_income_fifteen_day_ltv_hms, ad_income_thirty_day_ltv_hms, ad_income_one_day_roi, ad_income_two_day_roi, ad_income_three_day_roi, ad_income_seven_day_roi, ad_income_fifteen_day_roi, ad_income_thirty_day_roi, attribution_income_iaa, attribution_income_iap_normalized, ad_position_id, country)
                            VALUES
                            (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
                            ON DUPLICATE KEY UPDATE show_count=VALUES(show_count),
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
                            let r = sqlx::query(sql)
                                .bind(&advertiser.advertiser_id)
                                .bind(&vo.adgroup_id)
                                .bind(&vo.adgroup_name)
                                .bind(&vo.campaign_id)
                                .bind(&vo.campaign_name)
                                .bind(&vo.package_name)
                                .bind(&stat_datetime)
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
                                .bind(&vo.country)
                                .execute(pool).await;
                            match r {
                                Ok(v) => {},
                                Err(e) => {
                                    println!("azadmin.reports err {}", e);
                                }
                            }
                        }
                         */
        
                        let mut page_info = data.page_info;
                        page_info.page = page_info.page + 1;
                        return Some(page_info);
                        // println!("query_advertiser_reports page: {}, total_page: {}", page, &page_info.total_page);
                        // if page < page_info.total_page {
                        //     // self.query_advertiser_reports(pool, advertiser, start_date, end_date, page + 1, page_size).await;
                        // }
                    }
                    
                    
                    
                } else {
                    println!("query_advertiser_reports failed: {}", reports.message);
                    if reports.message.eq("token已过期") {
                        self.check_market_access_token(pool).await;
                        return page_info;
                        // self.query_advertiser_reports(pool, advertiser, start_date, end_date, page, page_size).await;
                    }
                }
            }
            
        }
        None
    }


    pub async fn query_ads_reports(&self, pool: &Pool<MySql>, today: &String) {
        println!();
        println!("==query_ads_reports start==");
        let rs = sqlx::query_as::<_, AdsToken>("SELECT * FROM ads_account")
        .fetch_all(pool)
        .await;
        
        match rs {
            Ok(v) => {
                for adv in v {
                    println!("query_ads_reports {}", adv.client_id);

                    let mut page_info: Option<EarningPageInfo> = None;
                    loop {
                        if let Some(access_token) = game_repository::get_ads_access_token(pool, &adv.client_id).await {
                            page_info = self.query_ads_reports_by_token(pool, today, &adv.client_id, &access_token, page_info, 1000).await;
                            if let Some(pi) = page_info {
                                if pi.page <= pi.total_page {
                                    page_info = Some(pi);
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                // let today = Local::now().format("%Y-%m-%d").to_string();
                self.calc_ads_daily_reports(pool, &today).await;
            },
            Err(e) => {
                println!("query_ads_reports err: {}", e);
            }
        }

        println!("==query_ads_reports end==");
        println!();
    }

    // #[async_recursion]
    async fn query_ads_reports_by_token(&self, pool: &Pool<MySql>, today: &String, client_id: &String, access_token: &String, page_info: Option<EarningPageInfo>, page_size: i32) -> Option<EarningPageInfo> {
        let page = match &page_info {
            Some(v) => v.page,
            None => 1
        };
        println!("query page {}", page);
        let rs = server_api::query_ads_reports_by_token(&access_token, &today, &today, page, page_size).await;
        if let Some(reports) = rs {
            if reports.code == "0" {
                let mut countries = HashSet::new();
                let mut apps = HashMap::new();
                let now = self.timestamp();
                for vo in reports.data.list {
                    if !countries.contains(&vo.country) {
                        countries.insert(vo.country.clone());
                    }
                    
                    apps.insert(vo.app_id.clone(), vo.app_name.clone());
                    let id = "".to_string() + &vo.app_id + "-" + &vo.stat_datetime.as_str() +"-"+&vo.placement_id;
                    let rs = sqlx::query("INSERT INTO azadmin.ads_earnings
                    (id,app_id,stat_datetime,click_through_rate,ad_type,click_count,placement_id,ad_requests_match_rate,app_name,earnings,ad_requests_show_rate,placement_name,matched_reached_ad_requests,show_count)
                    VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)
                    ON DUPLICATE KEY UPDATE 
                    click_through_rate=VALUES(click_through_rate),
                    click_count=VALUES(click_count),
                    ad_requests_match_rate=VALUES(ad_requests_match_rate),
                    earnings=VALUES(earnings),
                    ad_requests_show_rate=VALUES(ad_requests_show_rate),
                    matched_reached_ad_requests=VALUES(matched_reached_ad_requests),
                    show_count=VALUES(show_count)")
                        .bind(&id)
                        .bind(&vo.app_id)
                        .bind(&vo.stat_datetime)
                        .bind(&vo.click_through_rate)
                        .bind(&vo.ad_type)
                        .bind(&vo.click_count)
                        .bind(&vo.placement_id)
                        .bind(&vo.ad_requests_match_rate)
                        .bind(&vo.app_name)
                        .bind(&vo.earnings)
                        .bind(&vo.ad_requests_show_rate)
                        .bind(&vo.placement_name)
                        .bind(&vo.matched_reached_ad_requests)
                        .bind(&vo.show_count)
                        .execute(pool).await;
                    
                    match rs {
                        Ok(v) => {

                        },
                        Err(e) => {
                            println!("query_ads_reports_by_token insert err : {}", e);
                        }
                    }
                }

                if !apps.is_empty() {
                    self.create_app_if_not_exists(pool, client_id, apps).await;
                }

                for country_name in countries {
                    game_repository::save_country(pool, country_name).await;
                }
                
                println!("use {}", self.timestamp() - now);


                let mut page_info = reports.data.page_info;
                page_info.page = page_info.page + 1;
                return Some(page_info);
                // if page_info.page < page_info.total_page {
                //     self.query_ads_reports_by_token(pool, access_token, page_info.page + 1, page_size);
                // } else {
                //     println!("done");
                // }
                
            } else {
                println!("query_ads_reports failed: {}", reports.message);
                if reports.message.eq("token已过期") {
                    self.check_ads_access_token(pool).await;
                    return page_info;
                }
            }
        }
        None
    }

    async fn calc_ads_daily_reports(&self, pool: &Pool<MySql>, date: &String) {
        let rs = sqlx::query_as::<_, AdsDailyReport>("SELECT app_id, stat_datetime, SUM(earnings) as earnings from ads_earnings WHERE stat_datetime=? GROUP BY app_id,stat_datetime;")
            .bind(&date)
            .fetch_all(pool).await;
        match rs {
            Ok(list) => {
                for vo in list {
                    self.insert_or_update_daily_report(pool, &vo).await;
                }
            },
            Err(e) => {
                println!("calc_ads_daily_reports err : {}", e);
            }
        }
    }

    async fn insert_or_update_daily_report(&self, pool: &Pool<MySql>, vo: &AdsDailyReport) {
        let key = format!("{}-{}", &vo.app_id, &vo.stat_datetime);
        let rs = sqlx::query("UPDATE ads_daily_earnings_reports
        SET earnings=? WHERE `key`=?")
            .bind(&vo.earnings)
            .bind(&key)
            .execute(pool).await;

        match rs {
            Ok(v) => {
                if v.rows_affected() == 0 {
                    let rs = sqlx::query("INSERT INTO ads_daily_earnings_reports
                    (`key`, app_id, stat_datetime, earnings)
                    VALUES(?,?,?,?);
                    ")
                        .bind(&key)
                        .bind(&vo.app_id)
                        .bind(&vo.stat_datetime)
                        .bind(&vo.earnings)
                        .execute(pool).await;
                    
                    match rs {
                        Ok(v) => {},
                        Err(e) => {
                            println!("insert ads_daily_earnings_reports err {}", e);
                        }
                    }
                }
            },
            Err(e) => {
                println!("create_app_if_not_exists err {}", e);
            }
        }
    }

    async fn create_app_if_not_exists(&self, pool: &Pool<MySql>, client_id: &String, apps: HashMap<String, String>) {
        for kv in apps {
            let rs = sqlx::query("INSERT INTO apps (app_id, app_name, client_id) SELECT ?,?,? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM apps WHERE app_id=?)")
                        .bind(&kv.0)
                        .bind(&kv.1)
                        .bind(client_id)
                        .bind(&kv.0)
                        .execute(pool).await;
            match rs {
                Ok(v) => {},
                Err(e) => {
                    println!("create_app_if_not_exists err {}", e);
                }
            }
        }
        
    }


    pub async fn query_last_90_release_reports(&self, pool: &Pool<MySql>) {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let is_exist = game_repository::is_daily_query_task_executed(pool, &today).await;
        if !is_exist {
            println!();
            println!("==query_last_90_release_reports start==");
            let end = Local::now().checked_sub_days(Days::new(1)).unwrap();
            let start = end.checked_sub_days(Days::new(90)).unwrap();
            let start_date = &start.format("%Y-%m-%d").to_string();
            let end_date = &end.format("%Y-%m-%d").to_string();
            println!("{} {}", start_date, end_date);
            self.query_reports(pool, &start, &end).await;
            game_repository::done_daily_query_task(pool, &today).await;
            println!("==query_last_90_release_reports end==");
            println!();
        }
    }

    pub async fn save_app_icon(&self, pool: &Pool<MySql>, app_id: &String, icon: &String) {
        game_repository::save_app_icon(pool, app_id, icon).await;
    }

    pub async fn get_countries(&self, pool: &Pool<MySql>) -> Option<Vec<Country>> {
        game_repository::get_countries(pool).await
    }

    

    fn timestamp(&self) -> i64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_secs() as i64 * 1000i64 + (since_the_epoch.subsec_nanos() as f64 / 1_000_000.0) as i64
    }

    pub async fn check_package_app_id(&self, pool: &Pool<MySql>) {
        let rs = game_repository::get_one_unknown_package_name(pool).await;
        if let Some(list) = rs {
            for pkg in list {
                let accounts = game_repository::get_untry_connect_token(pool, &pkg.package_name).await;
                if let Some(accounts) = accounts {
                    for account in accounts {
                        let package_name = pkg.package_name.clone();
                        let mysql = pool.clone();
                        let service = self.clone();
                        let account_copy = account.clone();
                        // actix_rt::spawn(async move {
                            service.get_app_id_by_package_name(&mysql, &package_name, &account_copy).await
                        // });
                    }
                }
            }
            // pkg.package_name
        }
    }

    async fn get_app_id_by_package_name(&self, pool: &Pool<MySql>, package_name: &String, account: &ConnectToken) {
        if let Some(access_token) = &account.connect_access_token {
            let rs = server_api::query_package_app_id(&account.connect_client_id, access_token, package_name).await;
            if let Some(rs) = rs {
                if rs.ret.code == 0 {
                    if rs.appids.is_empty() {
                        game_repository::add_package_name_except_client_id(pool, &account.connect_client_id, package_name).await;
                    } else {
                        let vo = rs.appids.first();
                        if let Some(vo) = vo {
                            let app_id = &vo.value;
                            game_repository::update_app_package_name(pool, app_id, package_name).await;
                            game_repository::remove_unknown_package_name(pool, package_name).await;
                        }
                    }
                }
            }
        }
    }

    pub async fn query_umeng_apps(&self, pool: &Pool<MySql>) {
        let mut page = 1;
        let mut apps: Vec<UMApp> = vec![];
        loop {
            let rs = umeng_api::get_app_list(page).await;
            if let Some(mut rs) = rs {
                apps.append(&mut rs.appInfos);
                if page < rs.totalPage {
                    page = page + 1;
                } else {
                    break
                }
            } else {
                break
            }
        }

        let now = self.timestamp();
        for app in apps {
            game_repository::save_um_apps(pool, &app.appkey, &app.name).await;
        }
        println!("insert_umeng_apps use {}", self.timestamp() - now);
    }

    pub async fn query_last_30_umeng_retentions(&self, pool: &Pool<MySql>) {
        let end = Local::now().checked_sub_days(Days::new(1)).unwrap();
        let start = end.checked_sub_days(Days::new(30)).unwrap();
        let start_date = &start.format("%Y-%m-%d").to_string();
        let end_date = &end.format("%Y-%m-%d").to_string();


        let is_executed = game_repository::is_daily_task_executed(pool, &end_date, 3).await;

        if !is_executed {
            let now = self.timestamp();

            let apps = game_repository::get_um_apps_with_package_name(pool).await;
            if let Some(apps) = apps {
                for app in apps {
                    let rs = umeng_api::get_retentions(&app.appkey, start_date, end_date).await;
                    if let Some(info_list) = rs {
                        game_repository::save_app_umeng_retention(pool, &app.appkey, &info_list.retentionInfo).await;
                    }

                    let rs = umeng_api::get_duration(&app.appkey, end_date).await;
                    if let Some(rs) = rs {
                        game_repository::save_app_umeng_duration(pool, &app.appkey, end_date, rs.average).await;
                    }
                }
            }
            println!("query_last_30_umeng_retentions use {}", self.timestamp() - now);

            game_repository::execute_daily_task_done(pool, &end_date, 3).await;
            
        }
        // let ret = end.checked_sub_days(Days::new(1)).unwrap();
        // let ret_date = &ret.format("%Y-%m-%d").to_string();

        

    }

    pub async fn query_umeng_duration(&self, pool: &Pool<MySql>) {
        let today = Local::now().format("%Y-%m-%d").to_string();

        let is_executed = game_repository::is_daily_task_executed(pool, &today, 4).await;

        if !is_executed {
            let now = self.timestamp();

            let rs = game_repository::get_umeng_app_without_duration(pool).await;
            if let Some(rs) = rs {
                for app in rs {
                    let rs = umeng_api::get_duration(&app.appkey, &app.date).await;
                    if let Some(rs) = rs {
                        game_repository::save_app_umeng_duration(pool, &app.appkey, &app.date, rs.average).await;
                    }
                }
            }
            println!("query_umeng_duration use {}", self.timestamp() - now);

            game_repository::execute_daily_task_done(pool, &today, 4).await;
        }
        
    }

    pub async fn get_earnings_reports(&self, pool: &Pool<MySql>, params: &ReqQueryEarningReports) -> Option<Vec<ResAdsEarningReports>> {
        let mut sql = "SELECT a.*, b.app_name, c.remark FROM (
            SELECT app_id, SUM(earnings) AS earnings FROM ads_daily_earnings_reports ".to_string();
        let mut conds = vec![];
        if let Some(app_ids) = &params.app_ids {
            conds.push(format!("FIND_IN_SET(app_id, '{}')", app_ids));
        }
        if let Some(start_date) = &params.start_date {
            conds.push(format!("stat_datetime>='{}'", start_date));
        }
        if let Some(end_date) = &params.end_date {
            conds.push(format!("stat_datetime<='{}'", end_date));
        }
        if !conds.is_empty() {
            sql.push_str("WHERE ");
            sql.push_str(conds.join(" AND ").as_str());
        }
        sql.push_str(" GROUP BY app_id) a LEFT JOIN apps b ON a.app_id=b.app_id LEFT JOIN ads_account c ON b.client_id = c.client_id WHERE a.earnings > 0");

        // println!("{}", &sql);
        let rs = sqlx::query_as::<_, ResAdsEarningReports>(sql.as_str())
        .fetch_all(pool)
        .await;
        match rs {
            Ok(v) => Some(v),
            Err(e) => {
                println!("get_earnings_reports {}", e);
                None
            }
        }
    }

    pub async fn query_last_90_day_earning_reports(&self, pool: &Pool<MySql>) {
        let mut idx = 90;
        let mut datetime = Local::now();
        loop {
            datetime = datetime.checked_sub_days(Days::new(1)).unwrap();
            let date = datetime.format("%Y-%m-%d").to_string();
            // println!("query_last_90_day_earning_reports {}", &date);
            
            let is_executed = game_repository::is_daily_task_executed(pool, &date, 1).await;
            if !is_executed {
                self.query_ads_reports(pool, &date).await;
                game_repository::execute_daily_task_done(pool, &date, 1).await;
            }
            
            idx = idx - 1;
            if idx == 0 {
                break;
            }
        }
        // let today = Local::now().format("%Y-%m-%d").to_string();
        // let is_executed = game_repository::is_daily_task_executed(pool, &today, 1).await;
        // if !is_executed {
        //     let yesterday = Local::now().checked_sub_days(Days::new(1)).unwrap();
        //     let yesterday = yesterday.format("%Y-%m-%d").to_string();
        //     self.query_ads_reports(pool, &yesterday).await;
        //     game_repository::execute_daily_task_done(pool, &today, 1).await;
        // }
    }
    
}