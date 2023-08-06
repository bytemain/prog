use chrono::Utc;

pub fn get_current_timestamp() -> i64 {
    let now = Utc::now();
    now.timestamp()
}
