use crate::app::App;
use chrono::Utc;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Renders the loading screen with spinner and progress messages
pub fn render_loading(frame: &mut Frame, app: &App, area: Rect) {
    let loading_block = Block::default()
        .title("Loading Movies")
        .borders(Borders::ALL)
        .style(Style::default());

    // Create spinner animation (simple rotating character)
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner_idx = (Utc::now().timestamp_millis() / 100) as usize % spinner_chars.len();
    let spinner = spinner_chars[spinner_idx];

    let mut loading_text = vec![
        Line::from(format!("{} Loading movie data...", spinner)),
        Line::from(""),
    ];

    // Add recent loading messages (last 5)
    for message in app.loading_messages.iter().rev().take(5).rev() {
        loading_text.push(Line::from(message.clone()));
    }

    let loading_paragraph = Paragraph::new(loading_text)
        .block(loading_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(loading_paragraph, area);
}
