use chrono::{DateTime, Utc};

pub fn current_time() -> DateTime<Utc> {
    Utc::now()
}
