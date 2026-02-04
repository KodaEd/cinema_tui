use crate::app::App;
use chrono::Datelike;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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

    // Split the area into date header and movie list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Date header
            Constraint::Min(1),     // Movie list
        ])
        .split(area);

    // Render date header
    render_date_header(frame, app, chunks[0]);

    // Get filtered movies for selected date
    let movies = app.get_filtered_movies();
    
    let title = format!(
        "Movies ({} showing - Use ↑↓/jk to scroll, ←→/hl to change date)",
        movies.len()
    );
    
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

    frame.render_stateful_widget(list, chunks[1], &mut app.list_state);
}

/// Renders the date header showing current selected date
fn render_date_header(frame: &mut Frame, app: &App, area: Rect) {
    if app.available_dates.is_empty() {
        let paragraph = Paragraph::new(Text::styled(
            "No dates available",
            Style::default().fg(Color::Gray),
        ))
        .block(Block::default().borders(Borders::ALL).title("Dates"));
        frame.render_widget(paragraph, area);
        return;
    }

    let today = chrono::Local::now();
    
    // Calculate approximate space needed for horizontal display
    // Each date takes roughly: "Mon 02/04" = ~10 chars + 3 spacing = 13 chars per date
    let estimated_width_per_date = 13;
    let available_width = area.width.saturating_sub(4); // Account for borders and padding
    let total_estimated_width = app.available_dates.len() * estimated_width_per_date;

    // If we have space, show all dates horizontally
    if total_estimated_width <= available_width as usize {
        render_horizontal_dates(frame, app, area, &today);
    } else {
        // Fall back to single date display with position indicator
        render_single_date(frame, app, area, &today);
    }
}

/// Renders all dates horizontally with the selected one highlighted
fn render_horizontal_dates(frame: &mut Frame, app: &App, area: Rect, today: &chrono::DateTime<chrono::Local>) {
    let mut spans = Vec::new();
    
    for (i, date) in app.available_dates.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        
        let is_selected = i == app.selected_date_index;
        let is_today = date.year() == today.year()
            && date.month() == today.month()
            && date.day() == today.day();
        
        // Format: "Mon 02/04" or "Today" for current day
        let date_str = if is_today {
            "Today".to_string()
        } else {
            date.format("%a %m/%d").to_string()
        };
        
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else if is_today {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        spans.push(Span::styled(date_str, style));
    }
    
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Dates (←→ or h/l to navigate)")
        );
    
    frame.render_widget(paragraph, area);
}

/// Renders a single date with position indicator (fallback for narrow screens)
fn render_single_date(frame: &mut Frame, app: &App, area: Rect, today: &chrono::DateTime<chrono::Local>) {
    let date_text = if let Some(date) = app.get_selected_date() {
        let is_today = date.year() == today.year()
            && date.month() == today.month()
            && date.day() == today.day();
        
        let day_name = date.format("%A").to_string();
        let date_str = date.format("%B %d, %Y").to_string();
        
        let prefix = if is_today { "Today - " } else { "" };
        
        format!("{}{} ({})", prefix, day_name, date_str)
    } else {
        "No dates available".to_string()
    };

    let date_indicator = if app.available_dates.len() > 1 {
        format!(" [{}/{}]", app.selected_date_index + 1, app.available_dates.len())
    } else {
        String::new()
    };

    let full_text = format!("{}{}", date_text, date_indicator);

    let paragraph = Paragraph::new(Text::styled(
        full_text,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Selected Date")
    )
    .style(Style::default());

    frame.render_widget(paragraph, area);
}
