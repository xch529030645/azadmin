use std::{collections::HashMap, fs, str::FromStr, borrow::Cow};

use chrono::{Local, format::Item};
use reqwest::header::HeaderMap;
use serde::{Serialize, de};
use serde_json::{from_str, Value};
use sqlx::ColumnIndex;
use uuid::Uuid;

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


    let rs = client.post("https://ads.cloud.huawei.com/openapi/monetization/reports/v1/publisher").headers(headers).json(&data).timeout(std::time::Duration::from_secs(10)).send().await;
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


pub async fn create_campaign(access_token: &str, advertiser_id: &str, campaign_name: &str, daily_budget: i32) -> Option<ResCampaign> {
    println!("start create_campaign for {}", advertiser_id);
    let data = ReqCreateCampaign {
        advertiser_id: advertiser_id.to_string(), 
        campaign_name: campaign_name.to_string(), 
        product_type: "ANDROID_APP".to_string(),
        daily_budget
    };
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/promotion/campaign/create", "POST", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResCreateCampaign, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code == "200" {
                        println!("create_campaign success: {}", &v.data.campaign_id);
                        Some(v.data)
                    } else {
                        println!("create_campaign failed: {:?}", &v.message);
                        None
                    }
                },
                Err(e) => {
                    println!("create_campaign failed: {} {}", e, &txt);
                    None
                }
            }
        },
        None => None
    }
}

pub async fn create_adgroup(access_token: &str, campaign_id: &String, product_id: &String, ready_ad: &ReqReadyAd) -> Option<ResCreateAdgroupData> {
    println!("start create_adgroup: {} {}", campaign_id, product_id);
    let mut adgroup_begin_date: String;
    let mut adgroup_end_date: Option<String> = None;
    if ready_ad.promotion_date.is_empty() {
        adgroup_begin_date = Local::now().format("%Y-%m-%d").to_string();
    } else {
        adgroup_begin_date = ready_ad.promotion_date.first().unwrap().clone();
        adgroup_end_date = match ready_ad.promotion_date.get(1) {
            Some(v) => Some(v.clone()),
            None => None
        }
    }
    let data = ReqCreateAdgroup {
        advertiser_id: ready_ad.advertiser_id.clone(),
        adgroup_name: ready_ad.ad_name.clone(),
        campaign_id: campaign_id.clone(),
        targeting_package_id: ready_ad.audience_package,
        targeting_package_scope: "SHARE".to_string(),
        product_id: product_id.to_string(),
        adgroup_begin_date,
        adgroup_end_date,
        time_period_type: ready_ad.time_type.clone(),
        time_period: None,
        price_type: ready_ad.bid_type.clone(),
        price: ready_ad.bid.parse::<f32>().unwrap(),
        creative_size_id: ready_ad.position.clone()
    };
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/promotion/adgroup/create", "POST", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResCreateAdgroup, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code == "200" {
                        println!("create_adgroup success: {}", &v.data.adgroup_id);
                        Some(v.data)
                    } else {
                        None
                    }
                },
                Err(e) => {
                    println!("create_adgroup err: {} {}", e, txt);
                    None
                }
            }
        },
        None => None
    }
}


pub async fn create_creative(access_token: &str, advertiser_id: &String, adgroup_id: &i64, creative_name: &String, creative_size_sub_type: &String, creative_size: &String, title: &Option<String>, images: Vec<i64>, icon: Option<i64>, video: Option<i64>, corprate_name: &Option<String>) -> Option<i64> {
    let imagesStruct = if !images.is_empty() {
        let mut list: Vec<ReqAssetsStruct> = vec![];
        for imid in images {
            list.push(ReqAssetsStruct {
                file: ReqFileStruct {
                    asset_id: imid
                }
            })
        }
        Some(list)
    } else {
        None
    };

    let iconStruct = if let Some(iconId) = icon {
        Some(ReqAssetsStruct {
            file: ReqFileStruct {
                asset_id: iconId
            }
        })
    } else {
        None
    };

    let videoStruct = if let Some(videoId) = video {
        Some(ReqAssetsStruct {
            file: ReqFileStruct {
                asset_id: videoId
            }
        })
    } else {
        None
    };
    
    let corporate = if let Some(corprate_name) = corprate_name {
        Some(
            ReqTitleStruct {
                text: corprate_name.clone()
            }
        )
    } else {
        None
    };

    let title_element = if let Some(title) = title {
        Some(ReqTitleStruct {
            text: title.clone()
        })
    } else {
        None
    };

    let data = ReqCreateCreative {
        advertiser_id: advertiser_id.clone(),
        adgroup_id: adgroup_id.clone(),
        creative_name: creative_name.clone(),
        creative_size_sub_type: creative_size_sub_type.clone(),
        creative_size: creative_size.clone().replace("x", "*"),
        content_struct: ReqContentStruct {
            images: imagesStruct,
            icon: iconStruct,
            video: videoStruct,
            title: title_element,
            corporate
        }
    };

    let create_creative_param = serde_json::to_string_pretty(&data);
    println!("create_creative_param: {}", create_creative_param.unwrap());

    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/promotion/creative/create", "POST", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResCreateCreative, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code == "200"{
                        println!("create_creative success");
                        Some(v.data.creative_id)
                    } else {
                        None
                    }
                },
                Err(e) => {
                    println!("create_creative err: {} {}", e, txt);
                    None
                }
            }
        },
        None => None
    }
}

pub async fn create_product(access_token: &str, advertiser_id: &String, app_id: &String) -> Option<ResCreateProductData> {
    println!("start create_product: C{}", app_id);
    let data = ReqCreateProduct {
        advertiser_id: advertiser_id.clone(),
        product_type: "ANDROID_APP".to_string(),
        ag_app_type: "AG_APP_FOR_DISPLAY_NETWORK".to_string(),
        product_info: ReqProductInfo {
            app: ReqProductInfoAppInfo {
                app_id: format!("C{}", app_id)
            }
        }
    };
    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/promotion/product/create", "POST", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResCreateProduct, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code == "200"{
                        println!("create_product success");
                        Some(v.data)
                    } else {
                        None
                    }
                },
                Err(e) => {
                    println!("create_product err: {} {}", e, txt);
                    None
                }
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
        page_size: 50,
        filtering: ReqQueryAudienceFiltering {
            targeting_type: "TARGET_TYPE_APP".to_string()
        }
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

pub async fn upload_file(access_token: &String, advertiser_id: &String, aid: i32, file_hash_sha256: &String) -> Option<i64> {
    let params = serde_json::json!({
        "access_token": access_token,
        "advertiser_id": advertiser_id,
        "aid": aid,
        "file_hash_sha256": file_hash_sha256
    });
    let client: reqwest::Client = reqwest::Client::new();
    let rs = client.post(format!("http://127.0.0.1:10003/assets/inv/upload_assets?access_token={}&advertiser_id={}&aid={}", access_token, advertiser_id, aid)).json(&params).send().await;
    match rs {
        Ok(v) => {
            let txt = v.text().await.unwrap();
            let res: ResUploadAssetsData = serde_json::from_str(&txt).unwrap();
            if res.err == 0 {
                res.data
            } else {
                None
            }
        },
        Err(e) => {
            println!("upload_file err: {}", e);
            None
        }
    }
    
    /*
    let file_token = get_upload_token(access_token, advertiser_id).await;
    if let Some(file_token) = file_token {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Authorization", ("Bearer ".to_string()+access_token).parse().unwrap());

        let file_byte=std::fs::read(file_path).unwrap();
        let part= reqwest::multipart::Part::bytes(Cow::from(file_byte)).file_name(file_name.clone());
        let file_token_part = reqwest::multipart::Part::text(file_token);
        let asset_name_part = reqwest::multipart::Part::text(file_name.clone());
        let form = reqwest::multipart::Form::new()
            .part("file", part)
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
     */
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

pub async fn query_position_price(access_token: &String, advertiser_id: &String, creative_size_id: &String, price_type: &String) -> Option<ResFloorPriceData> {
    let data = ReqQueryPositionPrice {
        advertiser_id: advertiser_id.clone(),
        filtering: ReqQueryPositionPriceFilter {
            creative_size_id: creative_size_id.parse::<i64>().unwrap(),
            price_type: price_type.clone()
        }
    };
    
    let rs = curl("https://ads.cloud.huawei.com/ads/v1/tools/position_price/query", "GET", access_token, &data).await;
    match rs {
        Some(txt) => {
            let rs: Result<ResFloorPrice, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    if v.code.eq("200") {
                        Some(v.data)
                    } else {
                        println!("query_position_price err 2: {:?}", v.message);
                        None
                    }
                },
                Err(e) => {
                    println!("query_position_price err: {}", &txt);
                    None
                }
            }
        },
        None => None
    }
}



pub async fn query_campaigns(access_token: &String, advertiser_id: &String, page: i32) -> Option<(i64, Vec<Campaign>)> {
    println!("query_campaigns {} {}", advertiser_id, page);
    let params = serde_json::json!({
        "advertiser_id": advertiser_id,
        "page": page,
        "page_size": 50
    });

    let rs = curl("https://ads-dra.cloud.huawei.com/ads/v1/promotion/campaign/query", "GET", access_token, &params).await;
    match rs {
        Some(txt) => {
            let rs: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(txt.as_str());
            match rs {
                Ok(v) => {
                    let code = v.get("code")?.as_str()?;
                    if code.eq("200") {
                        let data = v.get("data")?;
                        let total = data.get("total")?.as_i64()?;
                        let list = data.get("data")?.as_array()?;
                        let mut ret = vec![];
                        for item in list {
                            let vo: Result<Campaign, serde_json::Error> = serde_json::from_value(item.to_owned());
                            if let Ok(vo) = vo {
                                ret.push(vo);
                            }
                        }
                        Some((total, ret))
                    } else {
                        let msg = v.get("message")?.as_str();
                        println!("query_campaigns err 2: {:?}", code);
                        None
                    }
                },
                Err(e) => {
                    println!("query_campaigns err: {}", &txt);
                    None
                }
            }
        },
        None => None
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

pub async fn send_download(aid: i32) -> Option<String> {
    let client: reqwest::Client = reqwest::Client::new();
    let rs = client.get(format!("http://127.0.0.1:8000/assets/qunhui/download?asset_id={}", aid)).send().await;
    match rs {
        Ok(v) => {
            let txt = v.text().await.unwrap();
            let res: ResDownloadResult = serde_json::from_str(&txt).unwrap();
            if res.err == 0 {
                res.path
            } else {
                None
            }
        },
        Err(e) => {
            println!("send_download err: {}", e);
            None
        }
    }
}


pub async fn download(url: &str) {
    let client = reqwest::Client::builder()
    .tls_built_in_root_certs(true)
    .danger_accept_invalid_certs(true)
    .use_rustls_tls()
    .build().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
    headers.insert("Accept-Language", "zh-CN,zh;q=0.9".parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Pragma", "no-cache".parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
    headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
    headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
    headers.insert("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert("sec-ch-ua", "\"Not.A/Brand\";v=\"8\", \"Chromium\";v=\"114\", \"Google Chrome\";v=\"114\"".parse().unwrap());
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.insert("sec-ch-ua-platform", "\"macOS\"".parse().unwrap());
    
    let rs = client.get(url).headers(headers).send().await;
    match rs {
        Ok(v) => {
            println!("ok");
            let bytes = v.bytes().await;
            if let Ok(bytes) = bytes {
                let uuid = Uuid::new_v4();
                let suffix = url.split(".").last().unwrap();
                let tmpfile = format!("tmp/{}.{}", uuid.to_string(), suffix);
                fs::write(tmpfile, bytes);
            }
        },
        Err(e) => {
            println!("download err: {:?}: {}", e.status(), e);
            // None
        }
    }

    
}

