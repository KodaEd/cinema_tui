use crate::app::ritz::{get_ritz_movies, get_ritz_movies_threaded};

use chrono::{DateTime, Datelike, Local, TimeZone};
use ratatui::widgets::ListState;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::fs;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CachedMovieData {
    pub movie_times: MovieTimes,
    pub last_updated: DateTime<Local>,
}

pub enum MovieFetchMessage {
    Progress(String),
    Complete(MovieTimes),
    Error(String),
}

pub enum CurrentScreen {
    Main,
    Movie,
    Date,
    Exiting,
}

pub struct App {
    pub ritz_movie_times: MovieTimes,
    pub current_screen: CurrentScreen,
    pub searching: bool,
    pub search_term: String,
    pub loading_movies: bool,
    pub loading_messages: Vec<String>,
    pub receiver: Option<mpsc::Receiver<MovieFetchMessage>>,
    pub selected_movie_index: usize,
    pub list_state: ListState,
    pub selected_date_index: usize,
    pub available_dates: Vec<DateTime<Local>>,
    pub last_updated: Option<DateTime<Local>>,
}

type MovieTimes = HashMap<String, Vec<DateTime<Local>>>;

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut app = Self {
            ritz_movie_times: HashMap::new(),
            current_screen: CurrentScreen::Main,
            searching: false,
            search_term: String::new(),
            loading_movies: false,
            loading_messages: Vec::new(),
            receiver: None,
            selected_movie_index: 0,
            list_state,
            selected_date_index: 0,
            available_dates: Vec::new(),
            last_updated: None,
        };

        // Try to load cached data
        app.load_cache();
        app
    }

    fn get_cache_path() -> PathBuf {
        let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("cinema_tui");
        fs::create_dir_all(&path).ok();
        path.push("movie_cache.json");
        path
    }

    fn load_cache(&mut self) {
        let cache_path = Self::get_cache_path();
        if let Ok(contents) = fs::read_to_string(&cache_path) {
            if let Ok(cached_data) = serde_json::from_str::<CachedMovieData>(&contents) {
                self.ritz_movie_times = cached_data.movie_times;
                self.last_updated = Some(cached_data.last_updated);
                self.update_available_dates();
            }
        }
    }

    pub fn save_cache(&self) {
        if let Some(last_updated) = self.last_updated {
            let cache_data = CachedMovieData {
                movie_times: self.ritz_movie_times.clone(),
                last_updated,
            };
            
            if let Ok(json) = serde_json::to_string_pretty(&cache_data) {
                let cache_path = Self::get_cache_path();
                fs::write(cache_path, json).ok();
            }
        }
    }

    pub fn get_last_updated_display(&self) -> String {
        match self.last_updated {
            Some(last_updated) => {
                let now = Local::now();
                let duration = now.signed_duration_since(last_updated);
                
                if duration.num_minutes() < 1 {
                    "Just now".to_string()
                } else if duration.num_minutes() < 60 {
                    format!("{} min ago", duration.num_minutes())
                } else if duration.num_hours() < 24 {
                    format!("{} hr ago", duration.num_hours())
                } else {
                    format!("{} days ago", duration.num_days())
                }
            }
            None => "Never".to_string(),
        }
    }

    pub fn is_update_recommended(&self) -> bool {
        if self.available_dates.is_empty() {
            return false; // No data, don't show warning
        }

        // Check if the earliest date has passed
        if let Some(earliest_date) = self.available_dates.first() {
            let now = Local::now();
            // Compare just the dates (ignore time)
            let earliest_date_only = earliest_date.date_naive();
            let today = now.date_naive();
            
            // If the earliest date is before today, recommend update
            if earliest_date_only < today {
                return true;
            }
        }

        false
    }

    pub fn fetch_movies(&mut self) {
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.loading_movies = true;
        self.loading_messages.clear();

        std::thread::spawn(move || {
            get_ritz_movies_threaded(sender);
        });
    }

    pub fn next_movie(&mut self) {
        let movie_count = self.get_filtered_movies().len();
        if movie_count == 0 {
            return;
        }

        self.selected_movie_index = (self.selected_movie_index + 1) % movie_count;
        self.list_state.select(Some(self.selected_movie_index));
    }

    pub fn previous_movie(&mut self) {
        let movie_count = self.get_filtered_movies().len();
        if movie_count == 0 {
            return;
        }

        if self.selected_movie_index == 0 {
            self.selected_movie_index = movie_count - 1;
        } else {
            self.selected_movie_index = self.selected_movie_index.saturating_sub(1);
        }
        self.list_state.select(Some(self.selected_movie_index));
    }

    pub fn get_sorted_movies(&self) -> Vec<(String, Vec<chrono::DateTime<chrono::Local>>)> {
        let mut movies: Vec<_> = self
            .ritz_movie_times
            .iter()
            .map(|(name, times)| (name.clone(), times.clone()))
            .collect();
        movies.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        movies
    }

    pub fn update_available_dates(&mut self) {
        let mut dates = HashSet::new();

        for times in self.ritz_movie_times.values() {
            for time in times {
                // Get date at midnight for comparison
                let date = time.date_naive().and_hms_opt(0, 0, 0).unwrap();
                let date_time = Local.from_local_datetime(&date).unwrap();
                dates.insert(date_time.timestamp());
            }
        }

        self.available_dates = dates
            .into_iter()
            .map(|timestamp| {
                DateTime::from_timestamp(timestamp, 0)
                    .unwrap()
                    .with_timezone(&Local)
            })
            .collect();

        self.available_dates.sort();

        // Reset to first date
        self.selected_date_index = 0;
    }

    pub fn next_date(&mut self) {
        if !self.available_dates.is_empty() {
            self.selected_date_index = (self.selected_date_index + 1) % self.available_dates.len();
            self.selected_movie_index = 0;
            self.list_state.select(Some(0));
        }
    }

    pub fn previous_date(&mut self) {
        if !self.available_dates.is_empty() {
            if self.selected_date_index == 0 {
                self.selected_date_index = self.available_dates.len() - 1;
            } else {
                self.selected_date_index = self.selected_date_index.saturating_sub(1);
            }
            self.selected_movie_index = 0;
            self.list_state.select(Some(0));
        }
    }

    pub fn get_selected_date(&self) -> Option<&DateTime<Local>> {
        self.available_dates.get(self.selected_date_index)
    }

    pub fn get_filtered_movies(&self) -> Vec<(String, Vec<chrono::DateTime<chrono::Local>>)> {
        let selected_date = match self.get_selected_date() {
            Some(date) => date,
            None => return Vec::new(),
        };

        let mut movies: Vec<_> = self
            .ritz_movie_times
            .iter()
            .filter_map(|(name, times)| {
                let filtered_times: Vec<DateTime<Local>> = times
                    .iter()
                    .filter(|time| {
                        time.year() == selected_date.year()
                            && time.month() == selected_date.month()
                            && time.day() == selected_date.day()
                    })
                    .copied()
                    .collect();

                if filtered_times.is_empty() {
                    None
                } else {
                    Some((name.clone(), filtered_times))
                }
            })
            .collect();

        movies.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        movies
    }
}
