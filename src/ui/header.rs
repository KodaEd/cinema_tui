use ratatui::{
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Renders the header/title block at the top of the screen
pub fn render_header(frame: &mut Frame, area: Rect) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled("Movies", Style::default())).block(title_block);

    frame.render_widget(title, area);
}
