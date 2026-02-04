use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Renders the header/title block at the top of the screen
pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let last_updated = app.get_last_updated_display();
    let update_recommended = app.is_update_recommended();
    
    // Calculate spacing to push "Last updated" to the right
    let title_text = "Cinema Showtimes";
    let update_text = if update_recommended {
        format!("⚠ Update recommended - Last: {}", last_updated)
    } else {
        format!("Last updated: {}", last_updated)
    };
    
    // Calculate padding needed (account for borders)
    let available_width = area.width.saturating_sub(4) as usize; // 2 for borders, 2 for padding
    let title_len = title_text.len();
    let update_len = update_text.chars().count(); // Use chars().count() for unicode
    let total_text_len = title_len + update_len;
    
    let line = if total_text_len < available_width {
        let spacing = available_width - total_text_len;
        
        let update_style = if update_recommended {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        Line::from(vec![
            Span::styled(title_text, Style::default()),
            Span::raw(" ".repeat(spacing)),
            Span::styled(update_text, update_style),
        ])
    } else {
        // If not enough space, just show title
        Line::from(Span::styled(title_text, Style::default()))
    };

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(line).block(title_block);

    frame.render_widget(title, area);
}
