
use chrono::Utc;
use chrono::DateTime;
// use serde::Serialize;

#[derive(Debug,Clone)]
pub struct TempWithTs {
    pub temp: i32,
    pub ts: DateTime<Utc>,
}