
use std::fs;

// use actix_multipart_derive::MultipartForm;
use actix_web::{Responder};
use serde::{Deserialize, Serialize};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};


#[derive(Serialize,Deserialize)]
pub struct ServerConfig {
    pub password: String,
    pub profiles: String
}

impl ServerConfig {
    pub fn read() -> ServerConfig {
        let data = fs::read_to_string("config.json").unwrap();
        let config: Result<ServerConfig, serde_json::Error> = serde_json::from_str(data.as_str());
        config.unwrap()
    }
}

#[derive(Serialize,Deserialize)]
pub struct MysqlConfig {
    pub password: String,
}


#[derive(Serialize,Deserialize)]
pub struct ReqLogin {
    pub username: String,
    pub password: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqQueryReports {
    pub package_name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub country: Option<String>,
    pub page: i32,
    pub len: i32,
    pub order_prop: Option<String>,
    pub order: Option<String>,
    pub group_by_country: bool,
    pub ads_accounts: Option<String>
}

#[derive(Serialize,Deserialize)]
pub struct ReqQueryEarningReports {
    pub app_ids: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct ResAdsEarningReports {
    pub earnings: Option<f64>,
    pub app_name: Option<String>,
    pub app_id: Option<String>,
    pub remark: Option<String>
}

#[derive(Serialize,Deserialize)]
pub struct ReqBindApp {
    pub package_name: String,
    pub app_id: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqBindUmKey {
    pub appkey: String,
    pub app_id: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqRoas {
    pub package_name: String,
    pub start_date: String,
    pub end_date: String,
    pub country: Option<String>
}

#[derive(Serialize,Deserialize)]
pub struct ReqUploadAppIcon {
    pub app_id: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqAddAdv {
    pub advertiser_id: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqAddGallery {
    pub client_id: String,
    pub client_secret: String,
    pub connect_client_id: Option<String>,
    pub connect_client_secret: Option<String>,
    pub remark: Option<String>
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(rename = "file")]
    pub files: Vec<TempFile>,
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct ResAdsReports {
    // pub id: i32,
    // pub app_id: String,
    // pub stat_datetime: Option<String>,
    pub earnings: Option<f64>,
    pub cost: f64,
    pub active: i32,
    pub iaa: f64,
    pub first_day_iaa: Option<f64>,
    pub app_name: Option<String>,
    pub package_name: Option<String>,
    pub duration: Option<i32>,
    pub r1: Option<f32>,
    pub remark: Option<String>
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct ResAdsReportsCount {
    pub count: i32
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct AdminInfo {
    pub id: i32,
    pub password: String
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct ResSumReports {
    pub cost: Option<f64>,
    pub active: Option<i32>,
    pub iaa: Option<f64>,
    pub earnings: Option<f64>,
    pub first_day_iaa: Option<f64>
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct AdsToken {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
    pub connect_client_id: Option<String>,
    pub connect_client_secret: Option<String>,
    pub connect_access_token: Option<String>,
}


#[derive(sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct ConnectToken {
    pub client_id: String,
    pub connect_client_id: String,
    pub connect_client_secret: String,
    pub connect_access_token: Option<String>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct AppGallery {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
    pub remark: Option<String>,
    pub connect_client_id: Option<String>,
    pub connect_client_secret: Option<String>,
    pub connect_access_token: Option<String>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct ReleaseToken {
    pub advertiser_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub client_id: Option<String>,
    pub secret: Option<String>
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct ReleaseClientToken {
    pub advertiser_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub client_id: String,
    pub secret: String
}


#[derive(Deserialize, Serialize)]
pub struct ResGetReports {
    pub list: Vec<ResAdsReports>,
    pub total_number: i32,
}


#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct AdsDailyReport {
    pub app_id: String,
    pub stat_datetime: String,
    pub earnings: f64,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct AdsDailyReleaseReport {
    pub package_name: String,
    pub stat_datetime: String,
    pub cost: f64,
    pub active: i32,
    pub iaa: f64,
    pub country: String,
    pub advertiser_id: Option<String>
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct AdsRoas {
    pub package_name: String,
    pub stat_datetime: String,
    pub record_datetime: String,
    pub cost: f64,
    pub active: i32,
    pub iaa: f64,
    pub earnings: Option<f64>
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct App {
    pub id: i32,
    pub app_name: String,
    pub app_id: String,
    pub icon: Option<String>,
    pub package_name: Option<String>,
    pub client_id: String,
    pub appkey: Option<String>,
}


#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Country {
    pub name: String,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct UnknownPackageName {
    pub package_name: String,
}


#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct UMApp {
    pub name: String,
    pub appkey: String,
    pub package_name: Option<String>
}

#[derive(Deserialize, Serialize)]
pub struct ResUMAppList {
    pub totalPage: i32,
    pub page: i32,
    pub appInfos: Vec<UMApp>
}

#[derive(Deserialize, Serialize)]
pub struct RetentionInfo {
    pub date: String,
    pub totalInstallUser: i32,
    pub retentionRate: Vec<f32>
}

#[derive(Deserialize, Serialize)]
pub struct ResRetentionInfo {
    pub retentionInfo: Vec<RetentionInfo>
}

#[derive(Deserialize, Serialize)]
pub struct ResUseDuration {
    pub average: i32
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct UMRetentionApp {
    pub appkey: String,
    pub date: String,
}


#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct AdsAccount {
    pub client_id: String,
    pub remark: String
}


#[derive(Deserialize, Serialize)]
pub struct RspErr<T> {
    pub err: i32,
    pub data: Option<T>
}
pub struct Results {}

impl Results {
    pub fn done<T>(rs: &Option<&T>) -> impl Responder
    where
    T: ?Sized + Serialize,
    {
        return match rs {
            Some(v) => {
                let rsp = RspErr {
                    err: 0,
                    data: Some(v)
                };
                serde_json::to_string(&rsp)
            }
            None => {
                let rsp = RspErr {
                    err: 1,
                    data: Some("login failed")
                };
                serde_json::to_string(&rsp)
            }
        }
    }

    pub fn err(code: i32) -> impl Responder {
        let rsp = RspErr {
            err: code,
            data: Some("")
        };
        serde_json::to_string(&rsp)
    }
}