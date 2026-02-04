mod app;
use app::App;

mod ui;

use std::error::Error;
use std::io;
use std::time::Duration;

use ratatui::Terminal;
use ratatui::crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::{Backend, CrosstermBackend};

use crate::app::{CurrentScreen, MovieFetchMessage};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend + 'static>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;
        
        // Check for messages from the background thread
        if let Some(receiver) = &app.receiver {
            match receiver.try_recv() {
                Ok(MovieFetchMessage::Progress(message)) => {
                    app.loading_messages.push(message);
                }
                Ok(MovieFetchMessage::Complete(movie_times)) => {
                    app.ritz_movie_times = movie_times;
                    app.last_updated = Some(chrono::Local::now());
                    app.update_available_dates();
                    app.save_cache();
                    app.loading_movies = false;
                    app.loading_messages.clear();
                    app.receiver = None;
                    app.selected_movie_index = 0;
                    app.list_state.select(Some(0));
                }
                Ok(MovieFetchMessage::Error(error)) => {
                    app.loading_messages.push(format!("Error: {}", error));
                    app.loading_movies = false;
                    app.receiver = None;
                }
                Err(_) => {
                    // No message available, continue
                }
            }
        }
        
        // Poll for events with a timeout to allow UI updates
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                // Handle search input when searching is active
                if app.searching {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.search_term.push(c);
                        }
                        KeyCode::Backspace => {
                            app.search_term.pop();
                        }
                        KeyCode::Enter => {
                            // TODO: Implement search functionality
                            app.searching = false;
                        }
                        KeyCode::Esc => {
                            app.searching = false;
                            app.search_term.clear();
                        }
                        _ => {}
                    }
                    continue;
                }

                match app.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('m') => {
                            app.searching = true;
                        }
                        KeyCode::Char('g') => {
                            if !app.loading_movies {
                                app.fetch_movies();
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.next_movie();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.previous_movie();
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            app.next_date();
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.previous_date();
                        }
                        _ => {}
                    },
                    CurrentScreen::Movie => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                    CurrentScreen::Date => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') => {
                            return Ok(());
                        }
                        KeyCode::Char('n') => {
                            // TODO switch to close modal
                            app.current_screen = CurrentScreen::Main;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}
