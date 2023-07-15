
use std::fs;

// use actix_multipart_derive::MultipartForm;
use actix_web::{Responder};
use serde::{Deserialize, Serialize};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};


#[derive(Serialize,Deserialize)]
pub struct ServerConfig {
    pub password: String,
    pub host: String,
    pub port: i32,
    pub profiles: String,
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
pub struct ReqChangePassword {
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
    pub ads_accounts: Option<String>,
    pub advertisers: Option<String>,
}

#[derive(Serialize,Deserialize)]
pub struct ReqQueryOverview {
    pub start_date: String,
    pub end_date: String
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
pub struct ReqAdvRemark {
    pub advertiser_id: String,
    pub remark: String
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
    pub password: String,
    pub prev: String,
    pub name: Option<String>,
    pub is_set_password: bool,
    pub url: String,
    pub company_id: i32
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct ResSumReports {
    pub cost: Option<f64>,
    pub active: Option<i32>,
    pub iaa: Option<f64>,
    pub earnings: Option<f64>,
    pub first_day_iaa: Option<f64>
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct ResOverview {
    pub cost: Option<f64>,
    pub earnings: Option<f64>,
    pub stat_datetime: String
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
    pub secret: Option<String>,
    pub remark: Option<String>,
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
    pub country: String
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct AdsDailyReleaseReportAdv {
    pub package_name: String,
    pub stat_datetime: String,
    pub cost: f64,
    pub active: i32,
    pub iaa: f64,
    pub country: String,
    pub advertiser_id: String
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

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct Admin {
    pub id: Option<i32>,
    pub username: String,
    pub password: String,
    pub name: String,
    pub role: i32,
}


#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct ReqSaveAdminAdvertiser {
    pub uid: i32,
    pub advertisers: Vec<String>
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct AdminAdvertiser {
    pub uid: i32,
    pub advertiser_id: String,
    pub remark: Option<String>
}

#[derive(Serialize,Deserialize)]
pub struct ResLogin {
    pub token: String,
    pub privileges: String,
    pub name: Option<String>,
    pub is_set_password: bool,
    pub url: String,
    pub company_id: i32
}


#[derive(Serialize,Deserialize)]
pub struct FormCreateAudience {
    pub advertiser_id: String,
    pub name: String,
    pub description: String,
    pub countries: Vec<String>,
}

#[derive(Serialize,Deserialize)]
pub struct FormAdvertiser {
    pub advertiser_id: String,
}

#[derive(Serialize,Deserialize)]
pub struct ReqQueryPositionDetail {
    pub advertiser_id: String,
    pub creative_size_id: String
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct AudiencePreview {
    pub targeting_id: i64,
    pub advertiser_id: String,
    pub targeting_name: String,
    pub targeting_description: Option<String>,
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct Position {
    pub creative_size_id: String,
    pub advertiser_id: String,
    pub content: String
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct PositionDetail {
    pub creative_size_id: String,
    pub detail: Option<String>
}

#[derive(Serialize,Deserialize)]
pub struct TaskQueryReports {
    pub advertiser_id: String,
    pub date: String
}


#[derive(Serialize,Deserialize)]
pub struct FormQueryAssets {
    pub asset_type: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Serialize,Deserialize)]
pub struct FormUpdateCollectionStatus {
    pub task_id: i32,
    pub enabled: bool
}

#[derive(Serialize,Deserialize)]
pub struct FormCollectionId {
    pub task_id: i32
}

#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct Assets {
    pub id: i32,
    pub assets_name: String,
    pub file_hash_sha256: String,
    pub file_url: Option<String>,
    pub asset_type: Option<String>,
    pub width: i32,
    pub height: i32,
    pub video_play_duration: Option<i32>,
    pub file_size: Option<i32>,
    pub file_format: Option<String>,
    pub local_path: Option<String>,
    pub cover_url: Option<String>,
}


#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct CollectionTask {
    pub id: i32,
    pub enabled: bool,
    pub remark: Option<String>,
    pub min_cost: f64,
    pub require_roas: f64,
    pub check_hour: i32,
    pub check_minute: i32,
    pub operation: i32,
    pub advertisers: Option<String>
}


#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct CampaignStat {
    pub iaa: f64,
    pub cost: f64,
    pub campaign_id: String,
    pub advertiser_id: String
}


#[derive(sqlx::FromRow, Serialize,Deserialize)]
pub struct CollectionExecuteRecords {
    pub id: i32,
    pub date: String,
    pub task_id: i32,
    pub operation: i32,
    pub campaign_id: String,
    pub create_time: String,
    pub cost: Option<f64>,
    pub iaa: Option<f64>,
}


#[derive(Serialize,Deserialize)]
pub struct FormUpdateCollectionAdvertisers {
    pub task_id: i32,
    pub advertisers: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqSaveCollection {
    pub id: Option<i32>,
    pub min_cost: Option<f64>,
    pub check_hour: Option<i32>,
    pub check_minute: Option<i32>,
    pub operation: i32,
    pub remark: Option<String>,
    pub require_roas: Option<f64>,
    pub advertisers: Option<String>
}

#[derive(Serialize,Deserialize)]
pub struct ReqDelCollection {
    pub id: i32
}



#[derive(Serialize,Deserialize)]
pub struct ReqWebQueryPositionPrice {
    pub advertiser_id: String,
    pub creative_size_id: String,
    pub price_type: String
}


#[derive(Serialize,Deserialize)]
pub struct ReqCreativeAssets {
    pub id: i32
}


#[derive(Serialize,Deserialize)]
pub struct ReqCreative {
    pub creative_name: String,
    pub creative_size_subtype: String,
    pub text: String,
    pub icons: Option<Vec<ReqCreativeAssets>>,
    pub images: Option<Vec<ReqCreativeAssets>>,
    pub videos: Option<Vec<ReqCreativeAssets>>,
    pub size: String
}


#[derive(Serialize,Deserialize)]
pub struct ReqReadyAd {
    pub ad_name: String,
    pub advertiser_id: String,
    pub app: i32,
    pub audience_package: i64,
    pub bid: String,
    pub bid_type: String,
    pub budget: String,
    pub campaign_name: String,
    pub date_type: i32,
    pub open_search: bool,
    pub position: String,
    pub promotion_date: Vec<String>,
    pub promotion_target: i32,
    pub promotion_time: Option<Vec<String>>,
    pub time_type: String,
    pub creatives: Vec<ReqCreative>
}


#[derive(Serialize,Deserialize)]
pub struct ReqCreateAds {
    pub request_id: String,
    pub ad_list: Vec<ReqReadyAd>
}


#[derive(Serialize,Deserialize)]
pub struct ReqSearchAssets {
    pub search_text: Option<String>,
    pub format: Option<String>,
    pub size: Option<String>,
    pub page: i32,
    pub len: i32
}


#[derive(Serialize,Deserialize)]
pub struct ResDownloadResult {
    pub err: i32,
    pub path: Option<String>,
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