use std::{collections::HashMap, fs};

use reqwest::header::HeaderMap;
use serde_json::from_str;

use crate::model::*;

use super::{response::*, req::*};


pub async fn get_access_token(authorization_code: &String, client_id: &str, client_secret: &str, redirect_uri: &str) -> Option<ResAccessToken> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());

    let mut data = HashMap::new();
    data.insert("grant_type", "authorization_code");
    data.insert("code", authorization_code);
    data.insert("client_id", client_id);
    data.insert("client_secret", client_secret);
    data.insert("redirect_uri", redirect_uri);


    let rs = client.post("https://oauth-login.cloud.huawei.com/oauth2/v2/token").headers(headers).form(&data).send().await;
    match rs {
        Ok(v) => {
            let at = v.json::<ResAccessToken>().await;
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("get_access_token err2: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            println!("get_access_token err: {}", e);
            None
        }
    }
    
}


pub async fn refresh_token(refresh_token: &str, client_id: &str, client_secret: &str) -> Option<ResAccessToken> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());

    let mut data = HashMap::new();
    data.insert("grant_type", "refresh_token");
    data.insert("client_id", client_id);
    data.insert("client_secret", client_secret);
    data.insert("refresh_token", refresh_token);


    let rs = client.post("https://oauth-login.cloud.huawei.com/oauth2/v2/token").headers(headers).form(&data).send().await;
    match rs {
        Ok(v) => {
            // println!("{:?}", &v.text().await.unwrap());
            let at = v.json::<ResAccessToken>().await;
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("get_access_token err2: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            println!("get_access_token err: {}", e);
            None
        }
    }
}

pub async fn get_ads_access_token(ads_token: &AdsToken) -> Option<ResAdsAccessToken> {
    println!("get_ads_access_token {}", &ads_token.client_id);
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());

    let mut data = HashMap::new();
    data.insert("grant_type", "client_credentials");
    data.insert("client_id", &ads_token.client_id);
    data.insert("client_secret", &ads_token.client_secret);


    let rs = client.post("https://oauth-login.cloud.huawei.com/oauth2/v3/token").headers(headers).form(&data).send().await;
    match rs {
        Ok(v) => {
            let at = v.json::<ResAdsAccessToken>().await;
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("get_access_token err2: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            println!("get_access_token err: {}", e);
            None
        }
    }
}

pub async fn get_connect_api_access_token(ads_token: &ConnectToken) -> Option<ResAdsAccessToken> {
    println!("get_connect_api_access_token {}", &ads_token.connect_client_id);
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let mut data = HashMap::new();
    data.insert("grant_type", "client_credentials");
    data.insert("client_id", &ads_token.connect_client_id);
    data.insert("client_secret", &ads_token.connect_client_secret);


    let rs = client.post("https://connect-api.cloud.huawei.com/api/oauth2/v1/token").headers(headers).json(&data).send().await;
    match rs {
        Ok(v) => {
            let txt = v.text().await.unwrap();
            let at: Result<ResAdsAccessToken, serde_json::Error> = serde_json::from_str(&txt);
            // let at = v.json::<ResAdsAccessToken>().await;
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("get_connect_api_access_token err2: {}", &txt);
                    None
                }
            }
        },
        Err(e) => {
            println!("get_connect_api_access_token err: {}", e);
            None
        }
    }
}


pub async fn query_reports(advertiser_id: &String, access_token: &str, start_date: &str, end_date: &str, page: i32, page_size: i32) -> Option<ResReports> {
    println!("api query_reports {}", advertiser_id);
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", ("Bearer ".to_string()+access_token).parse().unwrap());

    let data = ReqQueryAdGroupReport::create(advertiser_id.to_string(), "STAT_TIME_GRANULARITY_DAILY".to_string(), page, page_size, start_date.to_string(), end_date.to_string());
    // println!("{:?}", &data);
    let rs = client.post("https://ads-dra.cloud.huawei.com/openapi/v2/reports/adgroup/query").headers(headers).json(&data).send().await;
    match rs {
        Ok(v) => {
            // let txt = v.text().await.unwrap();
            // fs::write("/Volumes/MacintoshHD/Work/Rust/azadmin/res.txt", txt);
            // // println!("query_reports res {:?}", v.text().await.unwrap());
            // None
            let at = v.json::<ResReports>().await;
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("query_reports err: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            println!("query_reports err: {}", e);
            None
        }
    }
}

pub async fn query_ads_reports_by_token(access_token: &String, start_date: &String, end_date: &String, page: i32, page_size: i32) -> Option<ResEarningReports> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", ("Bearer ".to_string()+access_token).parse().unwrap());

    let mut data = ReqQueryAdEarningReport::create(page, page_size, start_date, end_date);
    data.add_filter("currency", "USD");


    let rs = client.post("https://ads.cloud.huawei.com/openapi/monetization/reports/v1/publisher").headers(headers).json(&data).send().await;
    match rs {
        Ok(v) => {
            let txt = v.text().await.unwrap();
            let at: Result<ResEarningReports, serde_json::Error> = serde_json::from_str(txt.as_str());
            // println!("query_reports res {:?}", v.text().await.unwrap());
            // let at = v.json::<ResEarningReports>().await;
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("query_ads_reports_by_token err: {}", txt);
                    None
                }
            }
        },
        Err(e) => {
            println!("query_ads_reports_by_token err: {}", e);
            None
        }
    }
}


pub async fn query_package_app_id(client_id: &String, access_token: &String, package_name: &String) -> Option<ResAppId> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", ("Bearer ".to_string()+access_token).parse().unwrap());
    headers.insert("client_id", client_id.parse().unwrap());


    let url = format!("https://connect-api.cloud.huawei.com/api/publish/v2/appid-list?packageName={}", package_name);
    let rs = client.get(url.as_str()).headers(headers).send().await;
    match rs {
        Ok(v) => {
            // println!("query_reports res {:?}", v.text().await.unwrap());
            let txt = v.text().await.unwrap();
            let at: Result<ResAppId, serde_json::Error> = serde_json::from_str(&txt);
            // let at = v.json::<ResAppId>().await;
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("query_package_app_id json err: {}", &txt);
                    None
                }
            }
        },
        Err(e) => {
            println!("query_package_app_id err: {}", e);
            None
        }
    }
}
