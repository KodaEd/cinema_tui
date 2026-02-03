use std::{collections::HashMap, ops::Add};

use chrono::{DateTime, Duration, Local, NaiveTime, Timelike};
use scraper::{Html, Selector};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Rating {
    pub source: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Movie {
    pub title: String,
    pub year: String,
    pub rated: String,
    pub released: String,
    pub runtime: String,
    pub genre: String,
    pub director: String,
    pub writer: String,
    pub actors: String,
    pub plot: String,
    pub language: String,
    pub country: String,
    pub awards: String,
    pub poster: String,
    pub ratings: Vec<Rating>,
    pub metascore: String,
    #[serde(rename = "imdbRating")]
    pub imdb_rating: String,
    #[serde(rename = "imdbVotes")]
    pub imdb_votes: String,
    #[serde(rename = "imdbID")]
    pub imdb_id: String,
    #[serde(rename = "Type")]
    pub media_type: String,
    pub dvd: String,
    pub box_office: String,
    pub production: String,
    pub website: String,
    pub response: String,
}

pub fn fetch_html(url: &str) -> Result<String, reqwest::Error> {
    reqwest::blocking::get(url)?.text()
}

pub fn parse_showtimes_from_html(html: &str) -> Vec<(String, Vec<String>)> {
    let document = Html::parse_document(html);
    let stack_sel = Selector::parse("li.Stack").expect("valid selector");
    let title_sel = Selector::parse("span.Title a").expect("valid selector");
    let time_sel = Selector::parse("span.Time").expect("valid selector");

    document
        .select(&stack_sel)
        .filter_map(|el| {
            let title_el = el.select(&title_sel).next()?;
            let movie_name = title_el.text().collect::<String>().trim().to_string();
            let times: Vec<String> = el
                .select(&time_sel)
                .map(|t| t.text().collect::<String>().trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if movie_name.is_empty() {
                return None;
            }
            Some((movie_name, times))
        })
        .collect()
}

pub fn get_dates_for_week() -> Vec<(chrono::DateTime<Local>, String)> {
    let mut dates: Vec<(chrono::DateTime<Local>, String)> = Vec::new();

    // Sets the time to the start of the day
    let today = Local::now()
        .date_naive()
        .and_time(NaiveTime::MIN)
        .and_local_timezone(Local)
        .unwrap();

    dates.push((today, "today".to_string()));
    dates.push((today + chrono::Days::new(1), "tomorrow".to_string()));

    for day_offset in 2..7 {
        let date = today + chrono::Days::new(day_offset);
        let day_name = date.format("%A").to_string().to_lowercase(); // "monday", "tuesday", etc.
        dates.push((date, day_name));
    }

    dates
}

pub fn get_offset_from_string(time_string: &str) -> i64 {
    let time = NaiveTime::parse_from_str(time_string, "%-I:%M %P").unwrap();
    (time.hour() as i64 * 60) + time.minute() as i64
}

pub struct App {
    // TODO find a way to make this persistent
    pub ritz_movie_times: MovieTimes,
    pub exit: bool,
}

type MovieTimes = HashMap<String, Vec<DateTime<Local>>>;

impl App {
    pub fn new() -> Self {
        Self {
            ritz_movie_times: HashMap::new(),
            exit: false,
        }
    }

    pub fn get_ritz_movies(&mut self) {
        let mut movie_times: HashMap<String, Vec<DateTime<Local>>> = HashMap::new();

        for (date, date_label) in get_dates_for_week() {
            let url = format!("https://www.ritzcinemas.com.au/now-showing/{}", date_label);
            let html = fetch_html(&url).unwrap();
            use std::{thread, time};

            // need to randomise this so we don't get blocked
            thread::sleep(time::Duration::from_secs(2));

            let showtimes = parse_showtimes_from_html(&html);

            for (movie_name, times) in showtimes {
                for time in times {
                    let offset = get_offset_from_string(&time);
                    let datetime = date.add(Duration::minutes(offset));

                    movie_times
                        .entry(movie_name.clone())
                        .or_insert(Vec::new())
                        .push(datetime);
                }
            }
        }

        self.ritz_movie_times = movie_times;
    }
}
