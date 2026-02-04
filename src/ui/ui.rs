use crate::app::{App, CurrentScreen};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use super::footer::render_footer;
use super::header::render_header;
use super::loading::render_loading;
use super::main_content::render_main_content;
use super::movie_detail::render_movie_detail;

/// Main UI rendering function that orchestrates all UI components
pub fn ui(frame: &mut Frame, app: &mut App) {
    // Create the main layout: header, content area, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(1),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());

    // Render header
    render_header(frame, app, chunks[0]);

    // Render content based on current screen
    match app.current_screen {
        CurrentScreen::MovieDetail => {
            render_movie_detail(frame, app, chunks[1]);
        }
        _ => {
            // Render main content area (loading screen or movie list)
            if app.loading_movies {
                render_loading(frame, app, chunks[1]);
            } else {
                render_main_content(frame, app, chunks[1]);
            }
        }
    }

    // Render footer with instructions
    render_footer(frame, app, chunks[2]);
}
