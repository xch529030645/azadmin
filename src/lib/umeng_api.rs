use std::collections::HashMap;

use crypto::hmac::Hmac;
use crypto::mac::Mac;

use crypto::sha1::Sha1;

use crate::model::{ResUMAppList, ResRetentionInfo, ResUseDuration};

const UM_KEY: &[u8] = b"iRbRCprXQJ";
const UM_APPID: &str = "7327107";

// const UM_KEY: &[u8] = b"q5LPisTuYs";
// const UM_APPID: &str = "3237580";

fn sign(text: &str) -> String {
    // let key: &[u8] = b"iRbRCprXQJ";
    let mut mac= Hmac::new(Sha1::new(), UM_KEY);
    mac.input(text.as_bytes());
    let result = mac.result();
    let code = result.code();
    code.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("").to_uppercase()
}


pub async fn get_app_list(page: i32) -> Option<ResUMAppList> {

    let sign_str = format!("param2/1/com.umeng.uapp/umeng.uapp.getAppList/{}page{}perPage100", UM_APPID, page);
    let _aop_signature = sign(sign_str.as_str());

    let client = reqwest::Client::new();

    let url = format!("https://gateway.open.umeng.com/openapi/param2/1/com.umeng.uapp/umeng.uapp.getAppList/{}?perPage=100&_aop_signature={}&page={}", UM_APPID, &_aop_signature, page);
    // println!("{}", url);
    let rs = client.get(url).send().await;
    match rs {
        Ok(v) => {
            let txt = v.text().await.unwrap();
            let at = serde_json::from_str(&txt);
            // let at = v.json::<ResUMAppList>().await;
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("get_app_list err2: {}", &txt);
                    None
                }
            }
        },
        Err(e) => {
            println!("get_app_list err: {}", e);
            None
        }
    }
}

pub async fn get_retentions(appkey: &str, from_date: &str, end_date: &str) -> Option<ResRetentionInfo> {
    let client = reqwest::Client::new();
    let sign_str = format!("param2/1/com.umeng.uapp/umeng.uapp.getRetentions/{}appkey{}endDate{}periodTypedailystartDate{}", UM_APPID, appkey, end_date, from_date);
    let _aop_signature = sign(sign_str.as_str());
    let url = format!("https://gateway.open.umeng.com/openapi/param2/1/com.umeng.uapp/umeng.uapp.getRetentions/{}?periodType=daily&endDate={}&appkey={}&_aop_signature={}&startDate={}", UM_APPID, end_date, appkey, _aop_signature, from_date);
    let rs = client.get(url).send().await;
    match rs {
        Ok(v) => {
            let txt = v.text().await.unwrap();
            let at = serde_json::from_str(&txt);
            // let at = v.json::<ResUMAppList>().await;
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("get_retentions err2: {}", &txt);
                    None
                }
            }
        },
        Err(e) => {
            println!("get_retentions err: {}", e);
            None
        }
    }
}


pub async fn get_duration(appkey: &str, date: &str) -> Option<ResUseDuration> {
    let client = reqwest::Client::new();
    let sign_str = format!("param2/1/com.umeng.uapp/umeng.uapp.getDurations/{}appkey{}date{}statTypedaily", UM_APPID, appkey, date);
    let _aop_signature = sign(sign_str.as_str());
    let url = format!("https://gateway.open.umeng.com/openapi/param2/1/com.umeng.uapp/umeng.uapp.getDurations/{}?date={}&statType=daily&appkey={}&_aop_signature={}", UM_APPID, date, appkey, _aop_signature);
    let rs = client.get(url).send().await;
    match rs {
        Ok(v) => {
            let txt = v.text().await.unwrap();
            let at = serde_json::from_str(&txt);
            match at {
                Ok(at) => Some(at),
                Err(e) => {
                    println!("get_duration err2: {}", &txt);
                    None
                }
            }
        },
        Err(e) => {
            println!("get_duration err: {}", e);
            None
        }
    }
}