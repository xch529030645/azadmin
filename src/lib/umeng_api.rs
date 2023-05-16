use std::collections::HashMap;

use crypto::hmac::Hmac;
use crypto::mac::Mac;

use crypto::sha1::Sha1;

use crate::model::{ResUMAppList, ResRetentionInfo, ResUseDuration};


fn sign(text: &str) -> String {
    let key: &[u8] = b"iRbRCprXQJ";
    let mut mac= Hmac::new(Sha1::new(), key);
    mac.input(text.as_bytes());
    let result = mac.result();
    let code = result.code();
    code.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("").to_uppercase()
}


pub async fn get_app_list(page: i32) -> Option<ResUMAppList> {

    let sign_str = format!("param2/1/com.umeng.uapp/umeng.uapp.getAppList/7327107page{}perPage100", page);
    let _aop_signature = sign(sign_str.as_str());

    let client = reqwest::Client::new();

    let url = format!("https://gateway.open.umeng.com/openapi/param2/1/com.umeng.uapp/umeng.uapp.getAppList/7327107?perPage=100&_aop_signature={}&page={}", &_aop_signature, page);
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
    let sign_str = format!("param2/1/com.umeng.uapp/umeng.uapp.getRetentions/7327107appkey{}endDate{}periodTypedailystartDate{}", appkey, end_date, from_date);
    let _aop_signature = sign(sign_str.as_str());
    let url = format!("https://gateway.open.umeng.com/openapi/param2/1/com.umeng.uapp/umeng.uapp.getRetentions/7327107?periodType=daily&endDate={}&appkey={}&_aop_signature={}&startDate={}", end_date, appkey, _aop_signature, from_date);
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
    let sign_str = format!("param2/1/com.umeng.uapp/umeng.uapp.getDurations/7327107appkey{}date{}statTypedaily", appkey, date);
    let _aop_signature = sign(sign_str.as_str());
    let url = format!("https://gateway.open.umeng.com/openapi/param2/1/com.umeng.uapp/umeng.uapp.getDurations/7327107?date={}&statType=daily&appkey={}&_aop_signature={}", date, appkey, _aop_signature);
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