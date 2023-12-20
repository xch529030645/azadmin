use std::{collections::HashMap, vec};

use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
pub struct AuthorizationCode {
    pub authorization_code: String,
    pub state: String
}



#[derive(Serialize,Deserialize, Debug)]
pub struct ReqQueryAdGroupReport {
    pub advertiser_id: String,
    pub time_granularity: String,
    pub page: i32,
    pub page_size: i32,
    pub start_date: String,
    pub end_date: String,
    pub time_line: String,
    pub group_by: Vec<String>,
    pub is_abroad: bool
}

impl ReqQueryAdGroupReport {
    pub fn create(advertiser_id: String, time_granularity: String, page: i32, page_size: i32, start_date: String, end_date: String) -> ReqQueryAdGroupReport {
        return ReqQueryAdGroupReport {
            advertiser_id,
            time_granularity,
            page,
            page_size,
            start_date,
            end_date,
            time_line: "STAT_REPORTING_TIME".to_string(),
            group_by:  vec!["COUNTRY".to_string(), "ADGROUP_ID".to_string()],
            is_abroad: true
        };
    }
}




#[derive(Serialize,Deserialize)]
pub struct ReqQueryAdEarningReport<'a> {
    pub filtering: HashMap<&'a str, &'a str>,
    pub group_by: Vec<String>,
    pub time_granularity: String,
    pub page: i32,
    pub page_size: i32,
    pub start_date: &'a str,
    pub end_date: &'a str
}

impl ReqQueryAdEarningReport<'_> {
    pub fn create<'a>(page: i32, page_size: i32, start_date: &'a str, end_date: &'a str) -> ReqQueryAdEarningReport<'a> {
        return ReqQueryAdEarningReport {
            filtering: HashMap::new(),
            group_by: vec!["STAT_BREAK_DOWNS_APP_ID".to_string(), "STAT_BREAK_DOWNS_PLACEMENT_ID".to_string()],
            time_granularity: "STAT_TIME_GRANULARITY_DAILY".to_string(),
            page,
            page_size,
            start_date,
            end_date
        };
    }

    pub fn add_filter<'a>(&'a mut self, key: &'static str, value: &'static str) {
        self.filtering.insert(key, value);
    }
}

#[derive(Serialize,Deserialize)]
pub struct ReqCreateCampaign {
    pub advertiser_id: String,
    pub campaign_name: String,
    pub product_type: String,
    pub daily_budget: i32
}


#[derive(Serialize,Deserialize)]
pub struct ReqProductInfoAppInfo {
    pub app_id: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqProductInfo {
    pub app: ReqProductInfoAppInfo
}

#[derive(Serialize,Deserialize)]
pub struct ReqCreateProduct {
    pub advertiser_id: String,
    pub product_type: String,
    pub ag_app_type: String,
    pub product_info: ReqProductInfo,

}

#[derive(Serialize,Deserialize)]
pub struct ReqQueryProduct<'a> {
    pub advertiser_id: &'a str,
    pub page: i32,
    pub page_size: i32,
    pub filtering: HashMap<&'a str, &'a str>
}

impl<'a> ReqQueryProduct<'a> {
    pub fn create(advertiser_id: &str, page: i32) -> ReqQueryProduct {
        let mut filtering = HashMap::new();
        filtering.insert("product_type", "ANDROID_APP");
        filtering.insert("ag_app_type", "AG_APP_FOR_DISPLAY_NETWORK");
        ReqQueryProduct {
            advertiser_id,
            page,
            page_size: 50,
            filtering
        }
    }
}


#[derive(Serialize,Deserialize)]
pub struct CustomLocation {
    pub value: Vec<String>
}

#[derive(Serialize,Deserialize)]
pub struct ReqCreateAudience<'a> {
    pub advertiser_id: &'a str,
    pub targeting_name: &'a str,
    pub targeting_description: &'a str,
    pub targeting_type: &'a str,
    pub current_custom_location_struct: CustomLocation,
    pub not_installed_apps_struct: CustomLocation,
    pub ai_target_flag: &'a str,
}


impl<'a> ReqCreateAudience<'a> {
    pub fn create(advertiser_id: &'a str, targeting_name: &'a str, targeting_description: &'a str, current_custom_location_struct: CustomLocation) -> ReqCreateAudience<'a> {
        ReqCreateAudience {
            advertiser_id,
            targeting_name,
            targeting_description,
            targeting_type: "TARGET_TYPE_APP",
            current_custom_location_struct,
            not_installed_apps_struct: CustomLocation { value: vec![String::from("true")] },
            ai_target_flag: "AI_TARGET_DISABLE"
        }
    }
}


#[derive(Serialize,Deserialize)]
pub struct ReqQueryAudienceFiltering {
    pub targeting_type: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqQueryAudience {
    pub advertiser_id: String,
    pub page: i32,
    pub page_size: i32,
    pub filtering: ReqQueryAudienceFiltering
}


#[derive(Serialize,Deserialize)]
pub struct ReqQueryAssets {
    pub advertiser_id: String,
    pub page: i32,
    pub page_size: i32,
    pub filtering: Option<HashMap<String, String>>
}


#[derive(Serialize,Deserialize)]
pub struct ReqQueryPositionDetailFilter {
    pub creative_size_id: String,
    pub product_type: String,
}

#[derive(Serialize,Deserialize)]
pub struct ReqQueryPositionDetail2 {
    pub advertiser_id: String,
    pub filtering: ReqQueryPositionDetailFilter,
}


#[derive(Serialize,Deserialize)]
pub struct ReqQueryPositionPriceFilter {
    pub creative_size_id: i64,
    pub price_type: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqQueryPositionPrice {
    pub advertiser_id: String,
    pub filtering: ReqQueryPositionPriceFilter
}






#[derive(Serialize,Deserialize)]
pub struct ReqCreateAdgroup {
    pub advertiser_id: String,
    pub adgroup_name: String,
    pub campaign_id: String,
    pub targeting_package_id: i64,
    pub targeting_package_scope: String, //SHARE
    pub product_id: String,
    pub adgroup_begin_date: String,
    pub adgroup_end_date: Option<String>,
    pub time_period_type: String, //TIME_PERIOD_ALL, TIME_PERIOD_DAY_SPECIFIC
    pub time_period: Option<String>,
    pub price_type: String,
    pub price: f32,
    pub creative_size_id: String
}


#[derive(Serialize,Deserialize)]
pub struct ReqFileStruct {
    pub asset_id: i64
}

#[derive(Serialize,Deserialize)]
pub struct ReqAssetsStruct {
    pub file: ReqFileStruct
}

#[derive(Serialize,Deserialize)]
pub struct ReqTitleStruct {
    pub text: String
}

#[derive(Serialize,Deserialize)]
pub struct ReqContentStruct {
    pub images: Option<Vec<ReqAssetsStruct>>,
    pub icon: Option<ReqAssetsStruct>,
    pub video: Option<ReqAssetsStruct>,
    pub title: Option<ReqTitleStruct>,
    pub corporate: Option<ReqTitleStruct>
}


#[derive(Serialize,Deserialize)]
pub struct ReqCreateCreative {
    pub advertiser_id: String,
    pub adgroup_id: i64,
    pub creative_name: String,
    pub creative_size_sub_type: String,
    pub creative_size: String,
    pub content_struct: ReqContentStruct
}
