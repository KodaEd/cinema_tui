use chrono::{NaiveTime, Timelike};

pub fn fetch_html(url: &str) -> Result<String, reqwest::Error> {
    reqwest::blocking::get(url)?.text()
}

pub fn get_offset_from_string(time_string: &str) -> i64 {
    let time = NaiveTime::parse_from_str(time_string, "%-I:%M %P").unwrap();
    (time.hour() as i64 * 60) + time.minute() as i64
}
