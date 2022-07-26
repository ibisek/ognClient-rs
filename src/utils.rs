use chrono::{Utc, Datelike, Timelike};
use chrono::format::strftime;

pub fn now() -> String {
    let now = Utc::now();
    // format!(
    //     "{:?}-{:02}-{:02} {:02}:{:02}:{:02}",
    //     now.year_ce().1,
    //     now.month(),
    //     now.day(),
    //     now.hour(),
    //     now.minute(),
    //     now.second(),
    // )

    format!("{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
}
