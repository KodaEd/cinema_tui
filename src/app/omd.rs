use serde::{Deserialize, Serialize};
use std::error::Error;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Welcome {
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
    pub welcome_type: String,
    #[serde(rename = "DVD")]
    pub dvd: String,
    pub box_office: String,
    pub production: String,
    pub website: String,
    pub response: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Rating {
    pub source: String,
    pub value: String,
}

/// Fetches movie details from the OMDb API
pub fn fetch_movie_details(movie_title: &str, api_key: &str) -> Result<Welcome, Box<dyn Error>> {
    let url = format!(
        "http://www.omdbapi.com/?apikey={}&t={}",
        api_key,
        urlencoding::encode(movie_title)
    );

    let response = reqwest::blocking::get(&url)?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }

    let movie_data: Welcome = response.json()?;
    
    // Check if the API returned an error (Response field will be "False")
    if movie_data.response == "False" {
        return Err(format!("Movie not found: {}", movie_title).into());
    }

    Ok(movie_data)
}

/// Downloads and prepares a movie poster for rendering
pub fn download_poster(poster_url: &str, picker: &Picker) -> Result<StatefulProtocol, Box<dyn Error>> {
    // Download the image
    let response = reqwest::blocking::get(poster_url)?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download poster: status {}", response.status()).into());
    }

    // Get the image bytes
    let bytes = response.bytes()?;
    
    // Decode the image
    let dyn_img = image::load_from_memory(&bytes)?;
    
    // Create the protocol for rendering
    let protocol = picker.new_resize_protocol(dyn_img);
    
    Ok(protocol)
}
