use std::collections::HashMap;
use std::ops::Add;
use std::sync::mpsc;

use crate::app::App;
use crate::app::utils::{fetch_html, get_offset_from_string};
use crate::app::MovieFetchMessage;
use chrono::Duration;
use chrono::{DateTime, Datelike, Local, NaiveTime, Weekday};
use rand::Rng;
use scraper::{Html, Selector};
use std::thread;
use std::time;

fn parse_showtimes_from_html(html: &str) -> Vec<(String, Vec<String>)> {
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

fn scrape_available_day_endpoints() -> Result<Vec<String>, reqwest::Error> {
    let html = fetch_html("https://www.ritzcinemas.com.au/now-showing")?;
    let document = Html::parse_document(&html);
    let link_sel =
        Selector::parse(".swiper-slide a[href*='/now-showing/']").expect("valid selector");

    let endpoints: Vec<String> = document
        .select(&link_sel)
        .filter_map(|el| {
            let href = el.value().attr("href")?;
            // Extract the last segment from href (e.g., "/now-showing/friday" -> "friday")
            let endpoint = href.strip_prefix("/now-showing/")?.to_string();
            // Filter out "all" endpoint
            if endpoint == "all" || endpoint.is_empty() {
                None
            } else {
                Some(endpoint)
            }
        })
        .collect();

    Ok(endpoints)
}

fn calculate_date_from_tag(tag: &str) -> DateTime<Local> {
    let today = Local::now()
        .date_naive()
        .and_time(NaiveTime::MIN)
        .and_local_timezone(Local)
        .unwrap();

    match tag {
        "today" => today,
        "tomorrow" => today + chrono::Days::new(1),
        _ => {
            // Parse weekday name
            let target_weekday = match tag.to_lowercase().as_str() {
                "monday" => Weekday::Mon,
                "tuesday" => Weekday::Tue,
                "wednesday" => Weekday::Wed,
                "thursday" => Weekday::Thu,
                "friday" => Weekday::Fri,
                "saturday" => Weekday::Sat,
                "sunday" => Weekday::Sun,
                _ => return today, // Fallback to today for unknown tags
            };

            let current_weekday = today.weekday();

            // Calculate days until target weekday
            let days_until = if current_weekday == target_weekday {
                // If it's the same day, return today (not next week)
                0
            } else {
                let current_num = current_weekday.num_days_from_monday();
                let target_num = target_weekday.num_days_from_monday();

                if target_num > current_num {
                    // Target is later this week
                    target_num - current_num
                } else {
                    // Target is next week
                    7 - current_num + target_num
                }
            };

            today + chrono::Days::new(days_until as u64)
        }
    }
}

fn get_dates_for_week() -> Vec<(chrono::DateTime<Local>, String)> {
    match scrape_available_day_endpoints() {
        Ok(endpoints) => endpoints
            .into_iter()
            .map(|tag| {
                let date = calculate_date_from_tag(&tag);
                (date, tag)
            })
            .collect(),
        Err(e) => {
            eprintln!(
                "Warning: Failed to scrape endpoints: {}. Using fallback dates.",
                e
            );
            // Fallback to hardcoded week if scraping fails
            let mut dates = Vec::new();
            let today = Local::now()
                .date_naive()
                .and_time(NaiveTime::MIN)
                .and_local_timezone(Local)
                .unwrap();

            dates.push((today, "today".to_string()));
            dates.push((today + chrono::Days::new(1), "tomorrow".to_string()));

            for day_offset in 2..7 {
                let date = today + chrono::Days::new(day_offset);
                let day_name = date.format("%A").to_string().to_lowercase();
                dates.push((date, day_name));
            }

            dates
        }
    }
}

pub fn get_ritz_movies_threaded(sender: mpsc::Sender<MovieFetchMessage>) {
    let mut movie_times: HashMap<String, Vec<DateTime<Local>>> = HashMap::new();

    let dates = match get_dates_for_week_result() {
        Ok(dates) => dates,
        Err(e) => {
            let _ = sender.send(MovieFetchMessage::Error(format!("Failed to get dates: {}", e)));
            return;
        }
    };

    for (date, date_label) in dates {
        let message = format!("Getting movie times for {}", date_label);
        let _ = sender.send(MovieFetchMessage::Progress(message));

        let url = format!("https://www.ritzcinemas.com.au/now-showing/{}", date_label);
        let html = match fetch_html(&url) {
            Ok(html) => html,
            Err(e) => {
                let _ = sender.send(MovieFetchMessage::Error(format!("Failed to fetch {}: {}", date_label, e)));
                return;
            }
        };

        // need to randomise this so we don't get blocked
        let mut rng = rand::thread_rng();
        let sleep_secs = rng.gen_range(1000..=2000);
        thread::sleep(time::Duration::from_millis(sleep_secs));

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

    let _ = sender.send(MovieFetchMessage::Complete(movie_times));
}

fn get_dates_for_week_result() -> Result<Vec<(chrono::DateTime<Local>, String)>, reqwest::Error> {
    let endpoints = scrape_available_day_endpoints()?;
    Ok(endpoints
        .into_iter()
        .map(|tag| {
            let date = calculate_date_from_tag(&tag);
            (date, tag)
        })
        .collect())
}

pub fn get_ritz_movies(app: &mut App) {
    let mut movie_times: HashMap<String, Vec<DateTime<Local>>> = HashMap::new();

    app.loading_movies = true;

    for (date, date_label) in get_dates_for_week() {
        let message = format!("Getting movie times for {}", date_label);
        app.loading_messages.push(message);

        let url = format!("https://www.ritzcinemas.com.au/now-showing/{}", date_label);
        let html = fetch_html(&url).unwrap();

        // need to randomise this so we don't get blocked
        let mut rng = rand::thread_rng();
        let sleep_secs = rng.gen_range(1000..=2000);
        thread::sleep(time::Duration::from_millis(sleep_secs));

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

    app.loading_movies = false;
    app.loading_messages.clear();
    app.ritz_movie_times = movie_times;
}
