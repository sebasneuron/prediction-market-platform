use chrono::{DateTime, Duration, Utc};

use crate::generated::common::Timeframe;

impl Timeframe {
    /// Get the duration for the time range
    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            Timeframe::OneHour => Some(Duration::hours(1)),
            Timeframe::SixHour => Some(Duration::hours(6)),
            Timeframe::OneDay => Some(Duration::days(1)),
            Timeframe::OneWeek => Some(Duration::weeks(1)),
            Timeframe::OneMonth => Some(Duration::days(30)), // Approximate month
            Timeframe::All => None,                          // No time limit
            Timeframe::Unspecified => None,                  // Unspecified timeframe
        }
    }
    pub fn as_db_interval_str(&self) -> &str {
        match self {
            Self::All => "100 years",
            Self::Unspecified => "100 years",
            Self::OneHour => "1 hour",
            Self::SixHour => "6 hours",
            Self::OneDay => "1 day",
            Self::OneWeek => "1 week",
            Self::OneMonth => "1 month",
        }
    }

    pub fn get_start_time(&self) -> Option<DateTime<Utc>> {
        self.to_duration().map(|duration| Utc::now() - duration)
    }

    pub fn to_sql_condition(&self) -> String {
        match self.get_start_time() {
            Some(start_time) => format!("ts >= '{}'", start_time.format("%Y-%m-%d %H:%M:%S")),
            None => "1=1".to_string(),
        }
    }

    pub fn to_parameterized_sql(&self) -> (String, Option<DateTime<Utc>>) {
        match self.get_start_time() {
            Some(start_time) => ("ts >= ?".to_string(), Some(start_time)),
            None => ("1=1".to_string(), None),
        }
    }
}
