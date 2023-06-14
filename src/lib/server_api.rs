use std::{collections::HashMap, fs, str::FromStr, borrow::Cow};

use reqwest::header::HeaderMap;
use serde::{Serialize, de};
use serde_json::{from_str, Value};

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
    let mut ret: Option<ResReports> = None;
    let mut times = 0;
    loop {
        let rs = client.post("https://ads-dra.cloud.huawei.com/openapi/v2/reports/adgroup/query").headers(headers.clone()).json(&data).send().await;
        match rs {
            Ok(v) => {
                // let txt = v.text().await.unwrap();
                // fs::write("/Volumes/MacintoshHD/Work/Rust/azadmin/res.txt", txt);
                // // println!("query_reports res {:?}", v.text().await.unwrap());
                // None
                let at = v.json::<ResReports>().await;
                match at {
                    Ok(at) => {
                        ret = Some(at);
                        break;
                    },
                    Err(e) => {
                        println!("query_reports json err: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("query_reports http err: {}", e);
            }
        }
        times = times + 1;
        if times > 1 {
            break;
        }
    }

    ret
    
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


pub async fn create_campaign(access_token: &str, advertiser_id: &str, campaign_name: &str, daily_budget: i32, sync_flow_resource_searchad: &str) -> Option<ResCreateCampaign> {
    let data = ReqCreateCampaign {
        advertiser_id: advertiser_id.to_string(), 
        campaign_name: campaign_name.to_string(), 
        product_type: "ANDROID_APP".to_string(),
        daily_budget, 
        sync_flow_resource_searchad: sync_flow_resource_searchad.to_string()
    };
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/promotion/campaign/create", "POST", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => Some(v),
                Err(e) => None
            }
        },
        None => None
    }
}

pub async fn query_products(access_token: &str, advertiser_id: &str, page: i32) -> Option<ResQueryProduct> {
    let data = ReqQueryProduct::create(advertiser_id, page);
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/promotion/product/query", "GET", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => Some(v),
                Err(e) => {
                    println!("query_products err: {}", e);
                    None
                }
            }
        },
        None => None
    }
}

pub async fn create_audience_package(access_token: &str, advertiser_id: &str, targeting_name: &String, targeting_description: &String, location: Vec<String>) -> Option<ResAudienceData> {
    let data = ReqCreateAudience::create(advertiser_id, targeting_name, targeting_description, CustomLocation { value: location });
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/tools/targeting_package/create", "POST", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResCreateAudience, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code.eq("200") {
                        Some(v.data)
                    } else {
                        None
                    }
                },
                Err(e) => {
                    println!("create_audience_package err: {}", e);
                    None
                }
            }
        },
        None => None
    }
}


pub async fn query_audience_package(access_token: &str, advertiser_id: &String, page: i32) -> Option<ResQueryAudienceData> {
    let data = ReqQueryAudience {
        advertiser_id: advertiser_id.clone(),
        page,
        page_size: 50
    };

    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/tools/targeting_package/query", "GET", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResQueryAudience, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code.eq("200") {
                        Some(v.data)
                    } else {
                        None
                    }
                },
                Err(e) => {
                    println!("query_audience_package err: {}", e);
                    None
                }
            }
        },
        None => None
    }
}


pub async fn query_position(access_token: &String, advertiser_id: &String) -> Option<String> {
    let mut data = HashMap::new();
    data.insert("advertiser_id", advertiser_id);
    curl("https://ads-dra.cloud.huawei.com/ads/v1/tools/position/query", "GET", access_token, &data).await
}

pub async fn query_assets(access_token: &String, advertiser_id: &String, filtering: Option<HashMap<String, String>>, page: i32) -> Option<ResQueryAssetsData> {
    let data = ReqQueryAssets {
        advertiser_id: advertiser_id.clone(), 
        page, 
        page_size: 50,
        filtering
    };
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/tools/creative_asset/query", "GET", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResQueryAssetsResult, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code.eq("200") {
                        Some(v.data)
                    } else {
                        None
                    }
                },
                Err(e) => {
                    println!("query_assets err: {}", e);
                    None
                }
            }
        },
        None => None
    }
}

pub async fn query_position_detail(access_token: &String, advertiser_id: &String, creative_size_id: &String) -> Option<String> {
    let data = ReqQueryPositionDetail2 {
        advertiser_id: advertiser_id.clone(), 
        filtering: ReqQueryPositionDetailFilter {
            creative_size_id: creative_size_id.clone(),
            product_type: "ANDROID_APP".to_string()
        }
    };
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/tools/position_detail/query", "GET", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<Value, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    let code = v.get("code").unwrap().as_str().unwrap();
                    if code.eq("200") {
                        let data = v.get("data").unwrap();
                        Some(data.to_string())
                    } else {
                        None
                    }
                },
                Err(e) => {
                    println!("query_assets err: {}", e);
                    None
                }
            }
        },
        None => None
    }
}

pub async fn upload_file(access_token: &String, advertiser_id: &String, file_path: &String, file_name: &String) -> Option<ResUploadAssetsData> {
    let file_token = get_upload_token(access_token, advertiser_id).await;
    if let Some(file_token) = file_token {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Authorization", ("Bearer ".to_string()+access_token).parse().unwrap());

        let file_byte=std::fs::read(file_path).unwrap();
        let part= reqwest::multipart::Part::bytes(Cow::from(file_byte)).file_name(file_name.clone());
        let file_token_part = reqwest::multipart::Part::text(file_token);
        let asset_name_part = reqwest::multipart::Part::text(file_name.clone());
        let form = reqwest::multipart::Form::new().part("file", part)
            .part("file_token", file_token_part)
            .part("asset_name", asset_name_part);
        let rs = reqwest::Client::new()
            .post("https://ads-dra.cloud.huawei.com/ads/v1/tools/creative_asset/create")
            .headers(headers)
            .multipart(form)
            .send()
            .await;
        match rs {
            Ok(v) => {
                let txt = v.text().await.unwrap();
                let res: Result<ResUploadAssets, serde_json::Error> = serde_json::from_str(&txt);
                match res {
                    Ok(v) => {
                        if v.code == "200" {
                            Some(v.data)
                        } else {
                            println!("upload_file 3{}", &txt);
                            None
                        }
                    },
                    Err(e) => {
                        println!("upload_file 2 {}, {}", e, &txt);
                        None
                    }
                }
            },
            Err(e) => {
                println!("upload_file: {}", e);
                None
            }
        }
    } else {
        println!("get file_token failed");
        None
    }
}

pub async fn get_upload_token(access_token: &String, advertiser_id: &String) -> Option<String> {
    let mut data = HashMap::new();
    data.insert("advertiser_id", advertiser_id);
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/tools/file/token/query", "GET", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResUploadToken, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code.eq("200") {
                        Some(v.data.file_token)
                    } else {
                        None
                    }
                },
                Err(e) => {
                    println!("get_upload_token err: {}", e);
                    None
                }
            }
        },
        None => None
    }
}

pub async fn update_campaign_status(access_token: &String, advertiser_id: &String, campangn_id: &String, campaign_status: &String) -> bool {
    /**
     * OPERATION_ENABLE
     * OPERATION_DISABLE
     * OPERATION_DELETE
     */
    let mut data = HashMap::new();
    data.insert("advertiser_id", advertiser_id);
    data.insert("campaign_id", campangn_id);
    data.insert("campaign_status", campaign_status);
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/promotion/campaign/update", "POST", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResCustom, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code.eq("200") {
                        true
                    } else {
                        println!("update_campaign_status err 2: {:?}", v.message);
                        false
                    }
                },
                Err(e) => {
                    println!("update_campaign_status err: {}", e);
                    false
                }
            }
        },
        None => false
    }
}

async fn curl<T: Serialize + ?Sized>(url: &str, method: &str, access_token: &str, data: &T) -> Option<String> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", ("Bearer ".to_string()+access_token).parse().unwrap());

    let rs = if method.eq("POST") {client.post(url)} else {client.get(url)}.headers(headers).json(&data).send().await;
    match rs {
        Ok(v) => {
            let at = v.text().await.unwrap();
            Some(at)
        },
        Err(e) => {
            println!("{} err: {}", url, e);
            None
        }
    }

}

pub async fn get(url: &String) -> Option<String> {
    let client = reqwest::Client::new();
    let rs = client.get(url.as_str()).send().await;
    match rs {
        Ok(v) => {
            let txt = v.text().await.unwrap();
            Some(txt)
        },
        Err(e) => {
            println!("get err: {}", e);
            None
        }
    }
}


