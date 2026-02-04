use crate::app::ritz::{get_ritz_movies, get_ritz_movies_threaded};

use chrono::{DateTime, Local};
use ratatui::widgets::ListState;
use std::collections::HashMap;
use std::sync::mpsc;

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
}

type MovieTimes = HashMap<String, Vec<DateTime<Local>>>;

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            ritz_movie_times: HashMap::new(),
            current_screen: CurrentScreen::Main,
            searching: false,
            search_term: String::new(),
            loading_movies: false,
            loading_messages: Vec::new(),
            receiver: None,
            selected_movie_index: 0,
            list_state,
        }
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
        let movie_count = self.ritz_movie_times.len();
        if movie_count == 0 {
            return;
        }
        
        self.selected_movie_index = (self.selected_movie_index + 1) % movie_count;
        self.list_state.select(Some(self.selected_movie_index));
    }

    pub fn previous_movie(&mut self) {
        let movie_count = self.ritz_movie_times.len();
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
        let mut movies: Vec<_> = self.ritz_movie_times
            .iter()
            .map(|(name, times)| (name.clone(), times.clone()))
            .collect();
        movies.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        movies
    }
}
