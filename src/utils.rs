use chrono::Utc;

pub fn now() -> String {
    let now = Utc::now();
    format!("{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
}
