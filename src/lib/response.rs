use serde::{Deserialize, Serialize};

use super::req::CustomLocation;

#[derive(Deserialize, Serialize)]
pub struct ResAccessToken {
    pub access_token: String,
    pub expires_in: i32,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String
}

#[derive(Deserialize, Serialize)]
pub struct ResAdsAccessToken {
    pub access_token: String,
    pub expires_in: i32,
}

#[derive(Deserialize, Serialize)]
pub struct PageInfo {
    pub page: i32,
    pub page_size: i32,
    pub total_num: i32,
    pub total_page: i32
}

#[derive(Deserialize, Serialize)]
pub struct ResReportVo {
    pub advertiser_id: String,
    pub adgroup_id: String,
    pub adgroup_name: String,
    pub campaign_id: String,
    pub campaign_name: String,
    pub package_name: String,
    pub stat_datetime: String,
    pub show_count: i32,
    pub click_count: i32,
    pub cpc: String,
    pub thousand_show_cost: String,
    pub cost: String,
    pub download_count: i32,
    pub download_cost: String,
    pub install_count: i32,
    pub install_cost: String,
    pub active_count_normalized: i32,
    pub active_cost_normalized: String,
    pub register_count: i32,
    pub register_cost: String,
    pub retain_count_normalized: i32,
    pub retain_cost_normalized: String,
    // pub activate_hms_count: i32,
    // pub activate_hms_cost: String,
    // pub retain_hms_count: i32,
    // pub retain_hms_cost: String,
    pub three_day_retain_count: i32,
    pub three_day_retain_cost: String,
    pub subscribe_count: i32,
    pub subscribe_cost: String,
    pub seven_day_retain_count: i32,
    pub seven_day_retain_cost: String,
    pub publisher_real_price_one_day: String,
    pub ad_income_one_day_ltv_hms: String,
    pub ad_income_two_day_ltv_hms: String,
    pub ad_income_three_day_ltv_hms: String,
    pub ad_income_seven_day_ltv_hms: String,
    pub ad_income_fifteen_day_ltv_hms: String,
    pub ad_income_thirty_day_ltv_hms: String,
    pub ad_income_one_day_roi: String,
    pub ad_income_two_day_roi: String,
    pub ad_income_three_day_roi: String,
    pub ad_income_seven_day_roi: String,
    pub ad_income_fifteen_day_roi: String,
    pub ad_income_thirty_day_roi: String,
    pub attribution_income_iaa: String,
    pub attribution_income_iap_normalized: String,
    pub ad_position_id: String,
    pub country: Option<String>

}

#[derive(Deserialize, Serialize)]
pub struct ResReportData {
    pub page_info: PageInfo,
    pub list: Vec<ResReportVo>
}

#[derive(Deserialize, Serialize)]
pub struct ResReports {
    pub code: String,
    pub message: String,
    pub data: Option<ResReportData>
}

#[derive(Deserialize, Serialize)]
pub struct ResEarningReportVo {
    pub country: String,
    pub click_through_rate: f32,
    pub ad_type: String,
    pub click_count: i32,
    pub reached_ad_requests: i32,
    pub placement_id: String,
    pub ad_requests_match_rate: f32,
    pub app_name: String,
    pub earnings: f32,
    pub ad_requests_show_rate: f32,
    pub placement_name: String,
    pub matched_reached_ad_requests: i32,
    pub show_count: i32,
    pub app_id: String,
    pub stat_datetime: String
}

#[derive(Deserialize, Serialize)]
pub struct EarningPageInfo {
    pub page: i32,
    pub page_size: i32,
    pub total_number: i32,
    pub total_page: i32
}

#[derive(Deserialize, Serialize)]
pub struct ResEarningReportData {
    pub page_info: EarningPageInfo,
    pub list: Vec<ResEarningReportVo>
}

#[derive(Deserialize, Serialize)]
pub struct ResEarningReports {
    pub code: String,
    pub message: String,
    pub data: ResEarningReportData
}

#[derive(Deserialize, Serialize)]
pub struct ResAppIdRet {
    pub code: i32,
    pub msg: String
}

#[derive(Deserialize, Serialize)]
pub struct ResAppIdVo {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Serialize)]
pub struct ResAppId {
    pub ret: ResAppIdRet,
    pub appids: Vec<ResAppIdVo>
}


#[derive(Deserialize, Serialize)]
pub struct ResCampaign {
    pub campaign_id: String
}

#[derive(Deserialize, Serialize)]
pub struct ResCreateCampaign {
    pub code: String,
    pub message: Option<String>,
    pub data: ResCampaign
}


#[derive(Deserialize, Serialize)]
pub struct ResProductAppInfo {
    pub icon_url: String,
    pub app_store_download_url: String,
    pub package_name: String,
    pub description: String,
    pub default_editor_desc: String,
    pub app_id: String,
    pub product_name: String
}

#[derive(Deserialize, Serialize)]
pub struct ResProductInfo {
    pub app: ResProductAppInfo
}

#[derive(Deserialize, Serialize)]
pub struct ResProduct {
    pub product_type: String,
    pub product_id: String,
    pub product_info: ResProductInfo
}

#[derive(Deserialize, Serialize)]
pub struct ResProductResults {
    pub total: i32,
    pub data: Vec<ResProduct>
}

#[derive(Deserialize, Serialize)]
pub struct ResQueryProduct {
    pub code: String,
    pub data: ResProductResults
}


#[derive(Deserialize, Serialize)]
pub struct ResAudienceData {
    pub targeting_id: i64,
}

#[derive(Deserialize, Serialize)]
pub struct ResCreateAudience {
    pub code: String,
    pub data: ResAudienceData
}

// ---- query audience
#[derive(Deserialize, Serialize)]
pub struct ResQueryAudienceInfo {
    pub targeting_id: i64,
    pub targeting_description: Option<String>,
    pub targeting_type: String,
    pub targeting_name: String,
    pub not_installed_apps_struct: Option<CustomLocation>,
    pub app_category_of_media_struct: Option<CustomLocation>,
    pub language_struct: Option<CustomLocation>,
    pub current_custom_location_struct: Option<CustomLocation>,
    pub gender_struct: Option<CustomLocation>,
    pub age_struct: Option<CustomLocation>,
    pub ai_target_flag: String
}

#[derive(Deserialize, Serialize)]
pub struct ResQueryAudienceData {
    pub total: i32,
    pub data: Vec<ResQueryAudienceInfo>
}

#[derive(Deserialize, Serialize)]
pub struct ResQueryAudience {
    pub code: String,
    pub data: ResQueryAudienceData
}




#[derive(Deserialize, Serialize)]
pub struct ResQueryAssets {
    pub asset_id: i64,
    pub asset_name: String,
    pub asset_type: String,
    pub file_url: String,
    pub width: i32,
    pub height: i32,
    pub video_play_duration: Option<i32>,
    pub file_size: i32,
    pub file_format: String,
    pub file_hash_sha256: String
}

#[derive(Deserialize, Serialize)]
pub struct ResQueryAssetsData {
    pub total: i32,
    pub creative_asset_infos: Vec<ResQueryAssets>
}

#[derive(Deserialize, Serialize)]
pub struct ResQueryAssetsResult {
    pub code: String,
    pub data: ResQueryAssetsData
}


#[derive(Deserialize, Serialize)]
pub struct ResUploadTokenData {
    pub file_token: String,
}

#[derive(Deserialize, Serialize)]
pub struct ResUploadToken {
    pub code: String,
    pub data: ResUploadTokenData
}

#[derive(Deserialize, Serialize)]
pub struct ResCustom {
    pub code: String,
    pub message: String
}



#[derive(Deserialize, Serialize)]
pub struct ResUploadAssetsData {
    pub url: String,
    pub asset_id: String
}

#[derive(Deserialize, Serialize)]
pub struct ResUploadAssets {
    pub code: String,
    pub data: ResUploadAssetsData
}