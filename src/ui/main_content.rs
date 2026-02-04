use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Renders the main content area showing the movie list or empty state
pub fn render_main_content(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.ritz_movie_times.is_empty() {
        let empty_block = Block::default()
            .title("No movies loaded - press 'g' to load")
            .borders(Borders::ALL)
            .style(Style::default());
        
        frame.render_widget(empty_block, area);
        return;
    }

    let title = format!(
        "Movies ({}  - Use ↑↓ or j/k to scroll)",
        app.ritz_movie_times.len()
    );

    let movies = app.get_sorted_movies();
    
    let items: Vec<ListItem> = movies
        .iter()
        .map(|(name, times)| {
            // Format times nicely
            let mut time_strings: Vec<String> = times
                .iter()
                .map(|t| t.format("%I:%M %p").to_string())
                .collect();
            time_strings.sort();
            
            let times_display = if time_strings.is_empty() {
                "No times available".to_string()
            } else {
                time_strings.join(", ")
            };

            // Create the movie line with name and times
            let content = vec![
                Line::from(vec![
                    Span::styled(
                        name.to_string(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!("  {}", times_display),
                        Style::default().fg(Color::Gray),
                    ),
                ]),
            ];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}
