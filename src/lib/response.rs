use serde::{Deserialize, Serialize};

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