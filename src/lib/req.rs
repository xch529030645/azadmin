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
            time_line: "STAT_REQUEST_TIME".to_string(),
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
            group_by: vec!["STAT_BREAK_DOWNS_APP_ID".to_string(), "STAT_BREAK_DOWNS_PLACEMENT_ID".to_string(), "STAT_BREAK_DOWNS_COUNTRY".to_string()],
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