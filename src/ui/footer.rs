use crate::app::{App, CurrentScreen};
use ratatui::{
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Returns the appropriate instruction text based on app state
fn get_instruction_text(app: &App) -> &'static str {
    if app.searching {
        "(Enter) to search, (Esc) to cancel, (q) to quit"
    } else if app.loading_movies {
        "Loading movies... (q) to quit"
    } else {
        match app.current_screen {
            CurrentScreen::Main => {
                if app.ritz_movie_times.is_empty() {
                    "(g) to load movies, (m) to search movies, (q) to quit"
                } else {
                    "(↑↓/jk) scroll, (←→/hl) change date, (g) refresh, (q) quit"
                }
            }
            CurrentScreen::Movie => "(d) to search dates, (q) to quit",
            CurrentScreen::Exiting => "(y) to confirm, (n) to cancel",
            _ => "",
        }
    }
}

/// Renders the footer with instructions at the bottom of the screen
pub fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let bottom_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let instruction_text = get_instruction_text(app);
    let bottom = Paragraph::new(Text::styled(instruction_text, Style::default()))
        .block(bottom_block);

    frame.render_widget(bottom, area);
}
