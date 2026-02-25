use chrono::{Datelike, NaiveDateTime, Timelike};
use uuid::Uuid;

pub fn to_cron_expression(datetime: NaiveDateTime) -> String {
    format!(
        "{} {} {} {} *",
        datetime.minute(),
        datetime.hour(),
        datetime.day(),
        datetime.month()
    )
}

pub enum CronJobName {
    CloseMarket(Uuid),
}

impl CronJobName {
    pub fn to_string(&self) -> String {
        match self {
            CronJobName::CloseMarket(market_id) => format!("close_market_job_{}", market_id),
        }
    }
}
