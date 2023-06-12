use std::{time::{SystemTime, UNIX_EPOCH}, collections::{HashMap, HashSet}, fs::{File, self}, io::{Write, Read}, borrow::BorrowMut, ops::Index, path};
use chrono::{Local, DateTime, Days};
use serde_json::Value;
use sqlx::{Pool, MySql, Row};

use crate::{lib::{server_api, req::{AuthorizationCode}, response::*, umeng_api}, model::*, auth, user_data::UserData, ctrl::promotion_service};

use super::game_repository;
// use async_recursion::async_recursion;

const CLIENT_ID: &str = "108128867";
const CLIENT_SECRET: &str = "9fae7c0bf88cd122d3fc87d1709fba4326d1f157a5d82a6aa1b4cf7123c09d54";
const REDIRECT_URL: &str = "https://dat.311419.cn/azadmin/auth";

#[derive(Clone)]
pub struct PromotionService {

}

impl PromotionService {
    pub fn create() -> PromotionService {
        PromotionService {}
    }

    pub async fn create_audience_package(&self, pool: &Pool<MySql>, param: FormCreateAudience) -> i32 {
        let access_token = game_repository::get_marketing_access_token(pool, &param.advertiser_id).await;
        if let Some(access_token) = access_token {
            let rs = server_api::create_audience_package(access_token.as_str(), &param.advertiser_id, &param.name, &param.description, param.countries.clone()).await;
            match rs {
                Some(data) => {
                    let c = &param.countries.join(",");
                    self.save_audience_package(pool, data.targeting_id, &param.advertiser_id, &param.name, &param.description, c, None, None, None, &"TARGET_TYPE_NOT_APP".to_string()).await;
                    0
                }
                None => 1
            }
        } else {
            1
        }
    }

    pub async fn sync_audience_package(&self, pool: &Pool<MySql>, advertiser_id: &String) -> i32 {
        let access_token = game_repository::get_marketing_access_token(pool, advertiser_id).await;
        if let Some(access_token) = access_token {
            let rs = server_api::query_audience_package(&access_token, advertiser_id, 1).await;
            match rs {
                Some(v) => {
                    for item in v.data {
                        let countries = if let Some(current_custom_location_struct) = item.current_custom_location_struct {
                            current_custom_location_struct.value.join(",")
                        } else {
                            "".to_string()
                        };
                        let gender = if let Some(gender_struct) = item.gender_struct {
                            gender_struct.value.join(",")
                        } else {
                            "".to_string()
                        };
                        let age = if let Some(age_struct) = item.age_struct {
                            age_struct.value.join(",")
                        } else {
                            "".to_string()
                        };
                        let language = if let Some(language_struct) = item.language_struct {
                            language_struct.value.join(",")
                        } else {
                            "".to_string()
                        };
                        self.save_audience_package(pool, item.targeting_id, advertiser_id, &item.targeting_name, &item.targeting_description.unwrap_or("".to_string()), &countries, Some(gender), Some(age), Some(language), &item.targeting_type).await;
                    }
                },
                None => {}
            }
        }
        0
    }

    async fn save_audience_package(&self, pool: &Pool<MySql>, targeting_id: i64, advertiser_id: &String, targeting_name: &String, targeting_description: &String, countries: &String, gender: Option<String>, age: Option<String>, language: Option<String>, targeting_type: &String) {
        let rs = sqlx::query("INSERT INTO azadmin.audience
        (targeting_id, advertiser_id, targeting_name, targeting_description, gender, age, country, language, targeting_type)
        VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE targeting_name=VALUES(targeting_name), targeting_description=VALUES(targeting_description), gender=VALUES(gender), age=VALUES(age), country=VALUES(country), language=VALUES(language), targeting_type=VALUES(targeting_type);
        ")
            .bind(targeting_id)
            .bind(advertiser_id)
            .bind(targeting_name)
            .bind(targeting_description)
            .bind(gender)
            .bind(age)
            .bind(countries)
            .bind(language)
            .bind(targeting_type)
            .execute(pool).await;
        // match rs {
        //     Ok(v) => true,
        //     Err(e) => false
        // }
    }


    pub async fn get_audience_package(&self, pool: &Pool<MySql>, advertiser_id: &String) -> Option<Vec<AudiencePreview>> {
        let rs = sqlx::query_as::<_, AudiencePreview>("SELECT targeting_id, advertiser_id, targeting_name, targeting_description FROM audience WHERE advertiser_id=?
        ")
            .bind(advertiser_id)
            .fetch_all(pool).await;
        match rs {
            Ok(v) => Some(v),
            Err(e) => {
                println!("get_audience_package err {}", e);
                None
            }
        }
    }

    pub async fn get_position(&self, pool: &Pool<MySql>, advertiser_id: &String) -> Option<Vec<Position>> {
        let rs = sqlx::query_as::<_, Position>("SELECT creative_size_id, advertiser_id, content FROM positions WHERE advertiser_id=?")
            .bind(advertiser_id)
            .fetch_all(pool).await;
        match rs {
            Ok(v) => {
                if v.is_empty() {
                    self.fetch_position(pool, advertiser_id).await
                } else {
                    Some(v)
                }
            },
            Err(e) => {
                println!("get_position err {}", e);
                self.fetch_position(pool, advertiser_id).await
            }
        }
    }

    async fn fetch_position(&self, pool: &Pool<MySql>, advertiser_id: &String) -> Option<Vec<Position>> {
        let access_token = game_repository::get_marketing_access_token(pool, advertiser_id).await;
        if let Some(access_token) = access_token {
            let txt = server_api::query_position(&access_token, advertiser_id).await;
            if let Some(txt) = txt {
                let json: Value = serde_json::from_str(txt.as_str()).unwrap();
                let code: &str = json.get("code").unwrap().as_str().unwrap();
                if code.eq("200") {
                    let data = json.get("data").unwrap();
                    let creative_size_info_list = data.get("creative_size_info_list").unwrap().as_array().unwrap();
                    let mut ret: Vec<Position> = Vec::new();
                    for info in creative_size_info_list {
                        let creative_size_id: String = info.get("creative_size_id").unwrap().as_i64().unwrap().to_string();
                        let content = info.to_string();
                        ret.push(Position {
                            creative_size_id: creative_size_id.clone(),
                            advertiser_id: advertiser_id.clone(),
                            content
                        })
                    }
                    if !ret.is_empty() {
                        self.save_position(pool, advertiser_id, &ret).await;
                    }
                    Some(ret)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
        
    }

    async fn save_position(&self, pool: &Pool<MySql>, advertiser_id: &String, positions: &Vec<Position>) {
        let mut sql = "INSERT INTO positions (creative_size_id, advertiser_id, content) VALUES ".to_string();
        let mut placeholders: Vec<&str> = vec![];
        for _ in positions {
            placeholders.push("(?,?,?)");
        }
        sql += placeholders.join(",").as_str();
        sql += " ON DUPLICATE KEY UPDATE content=VALUES(content)";

        let mut query = sqlx::query(sql.as_str());
        for info in positions {
            query = query.bind(&info.creative_size_id)
            .bind(advertiser_id)
            .bind(&info.content);
        }
        let rs = query.execute(pool).await;
        match rs {
            Ok(v) => {},
            Err(e) => {
                println!("save_position {}", e);
            }
        }
    }

    pub async fn fetch_assets(&self, pool: &Pool<MySql>) {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let is_executed = game_repository::is_daily_task_executed(pool, &today, 5).await;
        if !is_executed {
            let advs = game_repository::get_advertiser_tokens(pool).await;
            for adv in advs {
                self.fetch_assets_for_adv(pool, &adv).await;
            }

            game_repository::execute_daily_task_done(pool, &today, 5).await;
        }
    }

    async fn fetch_assets_for_adv(&self, pool: &Pool<MySql>, adv: &ReleaseToken) {
        println!("fetch_assets_for_adv: {}", adv.advertiser_id);
        if let Some(access_token) = &adv.access_token {
            // let mut page = 1;
            let total_page = Self::fetch_and_save_assets(pool, access_token, &adv.advertiser_id, 1).await;

            if total_page > 1 {
                let mut thread_headlers = vec![];
                for page in 2 .. total_page {
                    let mysql = pool.clone();
                    let advertiser_id = adv.advertiser_id.clone();
                    let access_token_copy = access_token.clone();

                    let handle = actix_rt::spawn(async move {
                        Self::fetch_and_save_assets(&mysql, &access_token_copy, &advertiser_id, page).await;
                    });
                    thread_headlers.push(handle);
                }

                for handle in thread_headlers {
                    if !handle.is_finished() {
                        handle.await;
                    }
                }
            }
            
            
            // loop {
            //     println!("fetch_assets_for_adv: {}, page: {}", adv.advertiser_id, page);
            //     let rs = server_api::query_assets(access_token, &adv.advertiser_id, page).await;
            //     if let Some(rs) = rs {
            //         let total_page = (rs.total as f32 / 50.0_f32).ceil() as i32;
            //         self.save_assets_for_adv(pool, &adv.advertiser_id, &rs.creative_asset_infos).await;
            //         if page >= total_page {
            //             break
            //         } else {
            //             page = page + 1;
            //         }
            //     } else {
            //         break
            //     }
            // }
        }
    }

    pub async fn save_assets_for_adv(pool: &Pool<MySql>, advertiser_id: &String, assets: &Vec<ResQueryAssets>) {
        for inv in assets {
            game_repository::save_assets(pool, advertiser_id, inv).await;
        }
    }

    pub async fn fetch_and_save_assets(pool: &Pool<MySql>, access_token: &String, advertiser_id: &String, page: i32) -> i32 {
        let rs = server_api::query_assets(access_token, advertiser_id, None, page).await;
        if let Some(rs) = rs {
            let total_page = (rs.total as f32 / 50.0_f32).ceil() as i32;
            Self::save_assets_for_adv(pool, advertiser_id, &rs.creative_asset_infos).await;
            total_page
        } else {
            0
        }
    }


    pub async fn query_position_detail(&self, pool: &Pool<MySql>, params: &ReqQueryPositionDetail) -> Option<PositionDetail> {
        let mut detail = game_repository::get_position_detail(pool, &params.creative_size_id).await;
        if detail.is_none() {
            let advertiser_id = &params.advertiser_id;
            let access_token = game_repository::get_marketing_access_token(pool, advertiser_id).await.unwrap();

            let rs = server_api::query_position_detail(&access_token, advertiser_id, &params.creative_size_id).await;
            if let Some(txt) = rs {
                game_repository::update_position_detail(pool, &params.creative_size_id, &txt).await;
                detail = Some(txt)
            }
        }
        
        
        if detail.is_some() {
            Some(PositionDetail {
                creative_size_id: params.creative_size_id.clone(),
                detail
            })
        } else {
            None
        }
    }

    pub async fn query_assets(&self, pool: &Pool<MySql>, req: &FormQueryAssets) -> Option<Vec<Assets>> {
        game_repository::query_assets(pool, req).await
    }


    pub async fn upload_file(&self, pool: &Pool<MySql>, aid: i32, advertiser_id: &String) -> Option<String> {
        let mut ret = None;
        let file_url = game_repository::get_assets_url(pool, aid).await;
        if let Some(file_url) = file_url {
            let names: Vec<&str> = file_url.split("/").collect();
            let name = names.last().unwrap().to_string();
            let tmp_path = "./tmp/".to_string() + name.as_str();
            let rs = self.download(&file_url, &tmp_path).await;
            if rs.is_ok() {
                let access_token = game_repository::get_marketing_access_token(pool, advertiser_id).await.unwrap();
                let rs =server_api::upload_file(&access_token, advertiser_id, &tmp_path, &name).await;
                if let Some(data) = rs {
                    let mut filtering = HashMap::new();
                    filtering.insert("asset_id".to_string(), data.asset_id.clone());
                    let rs = server_api::query_assets(&access_token, advertiser_id, Some(filtering), 1).await;
                    if let Some(rs) = rs {
                        Self::save_assets_for_adv(pool, advertiser_id, &rs.creative_asset_infos).await;
                    }
                    ret = Some(data.asset_id);
                }
                std::fs::remove_file(tmp_path);
            }
        }
        ret
    }

    pub async fn download(&self, file_url: &String, tmp_path: &String) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if !path::Path::new("./tmp").exists() {
            std::fs::create_dir("./tmp");
        }
        let mut file = match File::create(&tmp_path) {
            Err(why) => return Err(Box::new(why)),
            Ok(file) => file,
        };
        let req = reqwest::Client::new().get(file_url);
        let rep = req.send().await?.bytes().await?;
        // if !rep.status().is_success() {
        //     return;
        // }

        let data: std::result::Result<Vec<_>, _> = rep.bytes().collect();
        file.write_all(&data.unwrap())?;
        Ok(())
    }


    pub async fn get_collection_tasks(&self, pool: &Pool<MySql>) -> Option<Vec<CollectionTask>> {
        game_repository::get_collection_tasks(pool).await
    }
    
    pub async fn update_collection_tasks(&self, pool: &Pool<MySql>, param: &FormUpdateCollectionStatus) -> i32 {
        game_repository::update_collection_tasks(pool, param).await
    }
    
}