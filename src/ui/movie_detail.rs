use crate::app::App;
use chrono::Utc;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use ratatui_image::{StatefulImage, Resize, protocol::StatefulProtocol};
use tui_big_text::{BigText, PixelSize};

/// Renders the movie detail screen
pub fn render_movie_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    // Check if loading
    if app.loading_movie_detail {
        render_loading_state(frame, area);
        return;
    }

    // Check if API key is missing
    if app.omdb_api_key.is_none() {
        render_missing_api_key(frame, area);
        return;
    }

    // Check for errors
    if let Some(error) = &app.movie_detail_error {
        render_error_state(frame, area, error);
        return;
    }

    // Render movie details
    if app.selected_movie_detail.is_some() {
        render_movie_info(frame, area, app);
    } else {
        render_empty_state(frame, area);
    }
}

/// Renders loading state with spinner
fn render_loading_state(frame: &mut Frame, area: Rect) {
    let loading_block = Block::default()
        .title("Movie Details")
        .borders(Borders::ALL)
        .style(Style::default());

    // Create spinner animation
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner_idx = (Utc::now().timestamp_millis() / 100) as usize % spinner_chars.len();
    let spinner = spinner_chars[spinner_idx];

    let loading_text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} Fetching movie details from OMDb...", spinner),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
    ];

    let loading_paragraph = Paragraph::new(loading_text)
        .block(loading_block)
        .alignment(Alignment::Center);

    frame.render_widget(loading_paragraph, area);
}

/// Renders the poster section
fn render_poster_section(frame: &mut Frame, area: Rect, app: &mut App) {
    if app.loading_poster {
        // Show loading spinner
        let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let spinner_idx = (Utc::now().timestamp_millis() / 100) as usize % spinner_chars.len();
        let spinner = spinner_chars[spinner_idx];

        let loading_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                format!("{} Downloading poster...", spinner),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
        ];

        let loading_paragraph = Paragraph::new(loading_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Poster"));

        frame.render_widget(loading_paragraph, area);
    } else if let Some(protocol) = &mut app.poster_protocol {
        // Render the poster
        let image = StatefulImage::<StatefulProtocol>::default()
            .resize(Resize::Fit(None));

        let poster_block = Block::default()
            .borders(Borders::ALL)
            .title("Poster");

        let inner_area = poster_block.inner(area);
        frame.render_widget(poster_block, area);
        frame.render_stateful_widget(image, inner_area, protocol);
    } else {
        // Show placeholder
        let placeholder_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "No poster available",
                Style::default().fg(Color::Gray),
            )),
        ];

        let placeholder_paragraph = Paragraph::new(placeholder_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Poster"));

        frame.render_widget(placeholder_paragraph, area);
    }
}

/// Renders missing API key error with big text
fn render_missing_api_key(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Big text
            Constraint::Min(5),     // Instructions
        ])
        .split(area);

    // Big red text
    let big_text = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .lines(vec!["API KEY".into(), "REQUIRED!".into()])
        .alignment(Alignment::Center)
        .build();

    frame.render_widget(big_text, chunks[0]);

    // Instructions
    let instructions = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Please set your OMDb API key to view movie details",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "1. Get a free key at: http://www.omdbapi.com/apikey.aspx",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "2. Set environment variable: export OMDB_API_KEY=your_key_here",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "3. Restart the application",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press (Esc) or (b) to go back",
            Style::default().fg(Color::Gray),
        )),
    ];

    let instructions_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let instructions_paragraph = Paragraph::new(instructions)
        .block(instructions_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(instructions_paragraph, chunks[1]);
}

/// Renders error state
fn render_error_state(frame: &mut Frame, area: Rect, error: &str) {
    let error_block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .style(Style::default());

    let error_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Failed to fetch movie details",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            error,
            Style::default().fg(Color::Red),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This might happen if:",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            "- The movie title doesn't match OMDb database",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "- Network connection issues",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "- API rate limit reached",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press (Esc) or (b) to go back",
            Style::default().fg(Color::Gray),
        )),
    ];

    let error_paragraph = Paragraph::new(error_text)
        .block(error_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(error_paragraph, area);
}

/// Renders movie information
fn render_movie_info(frame: &mut Frame, area: Rect, app: &mut App) {
    // Get movie reference first to avoid borrow conflicts
    let movie = app.selected_movie_detail.as_ref().unwrap();
    
    let outer_block = Block::default()
        .title(format!("Movie Details - {}", movie.title))
        .borders(Borders::ALL)
        .style(Style::default());

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(30), // Poster section (fixed height)
            Constraint::Length(3),  // Title info
            Constraint::Min(10),    // Content
            Constraint::Length(1),  // Footer
        ])
        .split(inner_area);

    // Poster section
    render_poster_section(frame, chunks[0], app);

    // Get movie reference again for subsequent sections
    let movie = app.selected_movie_detail.as_ref().unwrap();

    // Title section
    render_title_section(frame, chunks[1], movie);

    // Main content
    render_content_section(frame, chunks[2], movie);

    // Footer
    let footer = Paragraph::new(Line::from(Span::styled(
        "Press (Esc) or (b) to go back, (q) to quit",
        Style::default().fg(Color::Gray),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}

/// Renders the title section with basic info
fn render_title_section(frame: &mut Frame, area: Rect, movie: &crate::app::omd::Welcome) {
    let title_info = vec![
        Line::from(vec![
            Span::styled(&movie.title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(format!("({})", movie.year), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Rating: ", Style::default().fg(Color::Gray)),
            Span::styled(&movie.rated, Style::default().fg(Color::White)),
            Span::raw(" | "),
            Span::styled("Runtime: ", Style::default().fg(Color::Gray)),
            Span::styled(&movie.runtime, Style::default().fg(Color::White)),
            Span::raw(" | "),
            Span::styled("Genre: ", Style::default().fg(Color::Gray)),
            Span::styled(&movie.genre, Style::default().fg(Color::White)),
        ]),
    ];

    let title_paragraph = Paragraph::new(title_info);
    frame.render_widget(title_paragraph, area);
}

/// Renders the main content section
fn render_content_section(frame: &mut Frame, area: Rect, movie: &crate::app::omd::Welcome) {
    let mut content = vec![];

    // Plot
    content.push(Line::from(Span::styled(
        "Plot:",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    content.push(Line::from(Span::styled(
        &movie.plot,
        Style::default().fg(Color::White),
    )));
    content.push(Line::from(""));

    // Director
    content.push(Line::from(vec![
        Span::styled("Director: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(&movie.director, Style::default().fg(Color::White)),
    ]));

    // Writer
    content.push(Line::from(vec![
        Span::styled("Writer: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(&movie.writer, Style::default().fg(Color::White)),
    ]));

    // Actors
    content.push(Line::from(vec![
        Span::styled("Actors: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(&movie.actors, Style::default().fg(Color::White)),
    ]));
    content.push(Line::from(""));

    // Ratings
    content.push(Line::from(Span::styled(
        "Ratings:",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // IMDb Rating
    if movie.imdb_rating != "N/A" {
        let rating_color = get_rating_color(&movie.imdb_rating);
        content.push(Line::from(vec![
            Span::raw("  IMDb: "),
            Span::styled(&movie.imdb_rating, Style::default().fg(rating_color).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({} votes)", movie.imdb_votes), Style::default().fg(Color::Gray)),
        ]));
    }

    // Metascore
    if movie.metascore != "N/A" {
        let rating_color = get_metascore_color(&movie.metascore);
        content.push(Line::from(vec![
            Span::raw("  Metascore: "),
            Span::styled(&movie.metascore, Style::default().fg(rating_color).add_modifier(Modifier::BOLD)),
        ]));
    }

    // Other ratings
    for rating in &movie.ratings {
        content.push(Line::from(vec![
            Span::raw(format!("  {}: ", rating.source)),
            Span::styled(&rating.value, Style::default().fg(Color::Yellow)),
        ]));
    }

    content.push(Line::from(""));

    // Additional info
    if movie.awards != "N/A" {
        content.push(Line::from(vec![
            Span::styled("Awards: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&movie.awards, Style::default().fg(Color::Yellow)),
        ]));
    }

    if movie.box_office != "N/A" {
        content.push(Line::from(vec![
            Span::styled("Box Office: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&movie.box_office, Style::default().fg(Color::White)),
        ]));
    }

    content.push(Line::from(vec![
        Span::styled("Language: ", Style::default().fg(Color::Gray)),
        Span::styled(&movie.language, Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::styled("Country: ", Style::default().fg(Color::Gray)),
        Span::styled(&movie.country, Style::default().fg(Color::White)),
    ]));

    let content_paragraph = Paragraph::new(content)
        .wrap(Wrap { trim: true });

    frame.render_widget(content_paragraph, area);
}

/// Renders empty state
fn render_empty_state(frame: &mut Frame, area: Rect) {
    let empty_block = Block::default()
        .title("Movie Details")
        .borders(Borders::ALL)
        .style(Style::default());

    let empty_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "No movie details available",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press (Esc) or (b) to go back",
            Style::default().fg(Color::Gray),
        )),
    ];

    let empty_paragraph = Paragraph::new(empty_text)
        .block(empty_block)
        .alignment(Alignment::Center);

    frame.render_widget(empty_paragraph, area);
}

/// Helper function to get color based on IMDb rating
fn get_rating_color(rating: &str) -> Color {
    if let Ok(score) = rating.parse::<f32>() {
        if score >= 7.0 {
            Color::Green
        } else if score >= 5.0 {
            Color::Yellow
        } else {
            Color::Red
        }
    } else {
        Color::White
    }
}

/// Helper function to get color based on Metascore
fn get_metascore_color(score: &str) -> Color {
    if let Ok(score_val) = score.parse::<i32>() {
        if score_val >= 70 {
            Color::Green
        } else if score_val >= 50 {
            Color::Yellow
        } else {
            Color::Red
        }
    } else {
        Color::White
    }
}
