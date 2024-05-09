//! Diffur

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph, Wrap},
};
use std::{
    io::{self},
    process::Command,
};
use tempfile::NamedTempFile;

/// The TUI state
struct App {
    /// Left temp file
    left: NamedTempFile,
    /// Right temp file
    right: NamedTempFile,
}

/// UpdateKind is an enum to represent the kind of update being performed to the app.
enum UpdateKind {
    Left,
    Right,
}

impl App {
    fn new() -> Self {
        Self {
            left: NamedTempFile::new().expect("Failed to create temp file"),
            right: NamedTempFile::new().expect("Failed to create temp file"),
        }
    }
}

fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

/// Starts the blocking event loop
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('a') => {
                    launch_editor(&app, UpdateKind::Left, terminal)
                        .map_err(|_| io::ErrorKind::BrokenPipe)?;
                }
                KeyCode::Char('b') => {
                    launch_editor(&app, UpdateKind::Right, terminal)
                        .map_err(|_| io::ErrorKind::BrokenPipe)?;
                }
                KeyCode::Char('c') => {
                    app.left.as_file().set_len(0)?;
                    app.right.as_file().set_len(0)?;
                }
                KeyCode::Char('d') => {
                    diff_files(&app, terminal).map_err(|_| io::ErrorKind::BrokenPipe)?;
                }
                KeyCode::Char('q') => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

/// Renders the UI
fn ui(f: &mut Frame, app: &App) {
    let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]);
    let [help_area, input_area] = vertical.areas(f.size());
    let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [input_left_area, input_right_area] = horizontal.areas(input_area);

    // Render the help message
    let help_message = help_widget();
    f.render_widget(help_message, help_area);

    // Get file contents
    let left_contents = std::fs::read_to_string(app.left.path()).unwrap_or_default();
    let right_contents = std::fs::read_to_string(app.right.path()).unwrap_or_default();

    // Render the left input box
    let input_left = Paragraph::new(left_contents)
        .style(Style::default())
        .wrap(Wrap { trim: false })
        .block(Block::bordered().title("Contents (Left)"));
    f.render_widget(input_left, input_left_area);

    // Render the right input box
    let input_right = Paragraph::new(right_contents)
        .style(Style::default())
        .wrap(Wrap { trim: false })
        .block(Block::bordered().title("Contents (Right)"));
    f.render_widget(input_right, input_right_area);
}

fn help_widget() -> Paragraph<'static> {
    let (msg, style) = (
        vec![
            "[q]".green().bold(),
            " Quit - ".into(),
            "[a]".green().bold(),
            " edit left - ".into(),
            "[b]".green().bold(),
            " edit right - ".into(),
            "[c]".green().bold(),
            " clear input - ".into(),
            "[d]".green().bold(),
            " diff text".into(),
        ],
        Style::default().add_modifier(Modifier::RAPID_BLINK),
    );
    let text = Text::from(Line::from(msg)).patch_style(style);
    Paragraph::new(text)
}

/// Launches the editor to edit a file in the [App].
fn launch_editor<B: Backend>(
    app: &App,
    update_kind: UpdateKind,
    terminal: &mut Terminal<B>,
) -> Result<()> {
    let mut stdout = io::stdout();

    // Exit the alternate screen to return to the normal terminal screen
    stdout.execute(LeaveAlternateScreen)?;

    // Disable raw mode to hand over the terminal to Vim properly
    disable_raw_mode()?;

    // Get the preferred editor from the environment
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    // Use the standard library to call Vim
    let path = match update_kind {
        UpdateKind::Left => app.left.path(),
        UpdateKind::Right => app.right.path(),
    };
    Command::new(editor).arg(path).status().expect("failed to edit");

    // Clear the screen and re-enable raw mode when Vim exits
    enable_raw_mode()?;

    // Enter the alternate screen again
    stdout.execute(EnterAlternateScreen)?;

    terminal.clear()?;
    terminal.draw(|f| ui(f, app))?;
    Ok(())
}

/// Launches `delta` to diff the two files in the [App].
fn diff_files<B: Backend>(app: &App, terminal: &mut Terminal<B>) -> Result<()> {
    let mut stdout = io::stdout();

    // Exit the alternate screen to return to the normal terminal screen
    stdout.execute(LeaveAlternateScreen)?;

    // Disable raw mode to hand over the terminal to Vim properly
    disable_raw_mode()?;

    // Use the standard library to call Vim
    Command::new("delta")
        .arg(app.left.path())
        .arg(app.right.path())
        .arg("--paging")
        .arg("always")
        .status()
        .expect("failed to diff");

    // Clear the screen and re-enable raw mode when Vim exits
    enable_raw_mode()?;

    // Enter the alternate screen again
    stdout.execute(EnterAlternateScreen)?;

    terminal.clear()?;
    terminal.draw(|f| ui(f, app))?;
    Ok(())
}
