use std::{time::{SystemTime, UNIX_EPOCH}, collections::{HashMap, HashSet}, ops::Sub};
use chrono::{Local, DateTime, Days};
use sqlx::{Pool, MySql, Row};

use crate::{lib::{server_api, req::{AuthorizationCode}, response::*}, model::*, auth};

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
        game_repository::add_app_gallery(pool, &param.client_id, &param.client_secret, &param.remark).await
    }

    pub async fn get_reports(&self, pool: &Pool<MySql>, params: &ReqQueryReports) -> Option<ResGetReports> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let yesterday = Local::now().checked_sub_days(Days::new(1)).unwrap().format("%Y-%m-%d").to_string();
        
        let mut sql = "SELECT a.package_name, SUM(a.cost) AS cost, CAST(SUM(a.active) AS SIGNED) AS active, SUM(a.iaa) AS iaa, DATE_FORMAT(a.stat_datetime , '%Y-%m-%d') as stat_datetime, b.app_name, c.earnings, SUM(d.iaa) as first_day_iaa 
        FROM ads_daily_release_reports a 
        LEFT JOIN apps b ON a.package_name = b.package_name 
        LEFT JOIN ads_daily_earnings_reports c ON b.app_id = c.app_id AND c.stat_datetime=a.stat_datetime 
        LEFT JOIN ads_daily_release_reports d ON a.package_name = d.package_name AND a.stat_datetime = d.stat_datetime and d.record_datetime = a.stat_datetime".to_string();
        let mut conds: Vec<String> = vec![];
        conds.push(format!("(a.record_datetime='{}' OR a.record_datetime='{}')", today, yesterday));
        if let Some(package_name) = &params.package_name {
            conds.push(format!("a.package_name='{}'", package_name));
        }
        if let Some(start_date) = &params.start_date {
            conds.push(format!("a.stat_datetime>='{}'", start_date));
        }
        if let Some(end_date) = &params.end_date {
            conds.push(format!("a.stat_datetime<='{}'", end_date));
        }
        if let Some(country) = &params.country {
            conds.push(format!("a.country='{}'", country));
        }

        if !conds.is_empty() {
            sql += format!(" WHERE {}", conds.join(" AND ")).as_str();
        }
        // if let Some(_) = &params.country {
        //     sql += " GROUP BY a.stat_datetime, a.package_name, b.app_name, c.earnings"
        // } else {
        //     sql += " GROUP BY a.stat_datetime, a.package_name, b.app_name, c.earnings"
        // }
        sql += format!(" ORDER BY a.stat_datetime DESC LIMIT {}, {}", params.page * params.len, params.len).as_str();
        // println!("{}", &sql);

        let rs = sqlx::query_as::<_, ResAdsReports>(sql.as_str())
        .fetch_all(pool)
        .await;
        let list = match rs {
            Ok(v) => Some(v),
            Err(e) => {
                println!("get_reports err {}", e);
                None
            }
        };

        sql = "SELECT COUNT(a.id) AS `count` FROM ads_daily_release_reports a".to_string();
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
    
    pub async fn get_apps(&self, pool: &Pool<MySql>) -> Option<Vec<App>> {
        let rs = sqlx::query_as::<_, App>("SELECT * FROM apps")
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

    pub async fn bind_app(&self, pool: &Pool<MySql>, param: &ReqBindApp) -> i32 {
        game_repository::bind_app(pool, param).await
    }

    pub async fn get_app_roas(&self, pool: &Pool<MySql>, param: &ReqRoas) -> Option<Vec<AdsRoas>> {
        game_repository::get_app_roas(pool, param).await
    }

    pub async fn login_admin(&self, pool: &Pool<MySql>, req: &ReqLogin) -> Option<String> {
        let rs = sqlx::query_as::<_, AdminInfo>("SELECT id, `password` FROM admin WHERE username=?")
        .bind(&req.username)
        .fetch_one(pool)
        .await;

        match rs {
            Ok(v) => {
                if v.password.eq(&req.password) {
                    let jwt = auth::create_jwt(&v.id);
                    Some(jwt)
                } else {
                    None
                }
            },
            Err(_) => None
        }
    }

    pub async fn check_access_token(&self, pool: &Pool<MySql>) {
        self.check_market_access_token(pool).await;
        self.check_ads_access_token(pool).await;
    }

    async fn check_market_access_token(&self, pool: &Pool<MySql>) {
        // println!();
        // println!("==check_market_access_token start==");
        let rs = sqlx::query_as::<_, ReleaseClientToken>("SELECT * from advertisers WHERE (ISNULL(expires_in) OR expires_in < UNIX_TIMESTAMP()*1000) AND NOT ISNULL(access_token)")
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
        let rs = sqlx::query_as::<_, AdsToken>("SELECT client_id, client_secret, access_token from ads_account WHERE ISNULL(expire_time) OR expire_time < UNIX_TIMESTAMP()*1000")
        .fetch_all(pool)
        .await;
        match rs {
            Ok(list) => {
                for ads_token in list {
                    let rs_token = server_api::get_ads_access_token(&ads_token).await;
                    if let Some(token) = rs_token {
                        self.save_ads_access_token(pool, &token, &ads_token).await;
                    }
                }
            },
            Err(e) => {
                println!("check_ads_access_token {}", e);
            }
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
                for adv in v {
                    if !days_range.is_empty() {
                        for days in days_range.clone() {
                            let adv_token_copy = adv.clone();
                            let mysql = pool.clone();
                            let service = self.clone();
                            let record_date = end_date.clone();
                            actix_rt::spawn(async move {
                                // let service = GameService::create();
                                for date in days {
                                    println!("query {}, from: {}, to: {}", adv_token_copy.advertiser_id, &date, &date);
                                    let mut page = 1;
                                    loop {
                                        let page_info = service.query_advertiser_reports(&mysql, &adv_token_copy, &date, &date, page, 100).await;
                                        if let Some(page_info) = page_info {
                                            if page < page_info.total_page {
                                                page = page + 1;
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                    service.calc_release_daily_reports(&mysql, &date, &record_date).await;
                                }
                                
                                // for date in days {
                                // }
                            });
                            
                        }
                    }
                }
                
            },
            Err(e) => {
                println!("query_reports err: {}", e);
            }
        }
        println!("==query_reports ends==");
        println!();
    }

    async fn calc_release_daily_reports(&self, pool: &Pool<MySql>, today: &String, record_date: &String) {
        let list = game_repository::calc_ads_daily_release_reports_by_date(pool, &today).await;
        if let Some(list) = list {
            for vo in list {
                game_repository::insert_or_update_daily_release_report(pool, &vo, record_date).await;
            }
        }
    }

    

    
    async fn query_advertiser_reports(&self, pool: &Pool<MySql>, advertiser: &ReleaseToken, start_date: &String, end_date: &String, page: i32, page_size: i32) -> Option<PageInfo> {
        println!("query_advertiser_reports page {}", page);
        if let Some(access_token) = &advertiser.access_token {
            let rs = server_api::query_reports(&advertiser.advertiser_id, access_token, &start_date, &end_date, page, page_size).await;
            if let Some(reports) = rs {
                if reports.code == "0" {
                    // let mut sqls = Vec::<String>::new();
                    let now = self.timestamp();
                    if let Some(data) = reports.data {
                        for vo in data.list {
                            let stat_datetime = "".to_string() + &vo.stat_datetime[0..4] + "-" + &vo.stat_datetime[4..6] + "-" + &vo.stat_datetime[6..8];
                            let sql = "INSERT INTO azadmin.reports
                            (advertiser_id, adgroup_id, adgroup_name, campaign_id, campaign_name, package_name, stat_datetime, show_count, click_count, cpc, thousand_show_cost, cost, download_count, download_cost, install_count, install_cost, active_count, active_cost, register_count, register_cost, retain_count, retain_cost, three_day_retain_count, three_day_retain_cost, subscribe_count, subscribe_cost, seven_day_retain_count, seven_day_retain_cost, publisher_real_price_one_day, ad_income_one_day_ltv_hms, ad_income_two_day_ltv_hms, ad_income_three_day_ltv_hms, ad_income_seven_day_ltv_hms, ad_income_fifteen_day_ltv_hms, ad_income_thirty_day_ltv_hms, ad_income_one_day_roi, ad_income_two_day_roi, ad_income_three_day_roi, ad_income_seven_day_roi, ad_income_fifteen_day_roi, ad_income_thirty_day_roi, attribution_income_iaa, attribution_income_iap_normalized, ad_position_id, country)
                            VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
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
                                // .bind(&vo.activate_hms_count)
                                // .bind(&vo.activate_hms_cost)
                                // .bind(&vo.retain_hms_count)
                                // .bind(&vo.retain_hms_cost)
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
                            // sqls.push(sql);
                        }
        
                        println!("query_advertiser_reports use {}", self.timestamp() - now);
        
                        // let page_info = &data.page_info;
                        return Some(data.page_info);
                        // println!("query_advertiser_reports page: {}, total_page: {}", page, &page_info.total_page);
                        // if page < page_info.total_page {
                        //     // self.query_advertiser_reports(pool, advertiser, start_date, end_date, page + 1, page_size).await;
                        // }
                    }
                    
                    
                    // let cmds = sqls.join("\n");
                    // let r = sqlx::query(cmds.as_str()).execute(pool).await;
                    
                } else {
                    println!("query reports failed: {}", reports.message);
                }
            }
            
        }
        None
    }


    pub async fn query_ads_reports(&self, pool: &Pool<MySql>) {
        println!();
        println!("==query_ads_reports start==");
        let rs = sqlx::query_as::<_, AdsToken>("SELECT * FROM ads_account")
        .fetch_all(pool)
        .await;
        
        match rs {
            Ok(v) => {
                for adv in v {
                    println!("query_ads_reports {}", adv.client_id);
                    if let Some(access_token) = adv.access_token {
                        let mut page = 1;
                        loop {
                            let page_info = self.query_ads_reports_by_token(&pool, &access_token, page, 100).await;
                            if let Some(page_info) = page_info {
                                if page < page_info.total_page {
                                    page = page + 1;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
                let today = Local::now().format("%Y-%m-%d").to_string();
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
    async fn query_ads_reports_by_token(&self, pool: &Pool<MySql>, access_token: &String, page: i32, page_size: i32) -> Option<EarningPageInfo> {
        println!("query page {}", page);
        let today = Local::now().format("%Y-%m-%d").to_string();
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
                    self.create_app_if_not_exists(pool, apps).await;
                }

                for country_name in countries {
                    game_repository::save_country(pool, country_name).await;
                }
                
                println!("use {}", self.timestamp() - now);


                let page_info = reports.data.page_info;
                return Some(page_info);
                // if page_info.page < page_info.total_page {
                //     self.query_ads_reports_by_token(pool, access_token, page_info.page + 1, page_size);
                // } else {
                //     println!("done");
                // }
                
            } else {
                println!("query reports failed: {}", reports.message);
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
                println!("p_calc_ads_daily_reports err : {}", e);
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

    async fn create_app_if_not_exists(&self, pool: &Pool<MySql>, apps: HashMap<String, String>) {
        for kv in apps {
            let rs = sqlx::query("INSERT INTO apps (app_id, app_name) SELECT ?,? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM apps WHERE app_id=?)")
                        .bind(&kv.0)
                        .bind(&kv.1)
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
}