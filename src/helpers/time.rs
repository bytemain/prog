use chrono::naive::NaiveDateTime;

pub fn get_current_timestamp() -> NaiveDateTime {
    chrono::Utc::now().naive_utc()
}
