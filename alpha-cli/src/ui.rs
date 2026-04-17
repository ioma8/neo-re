use std::{
    io::{self, Stdout},
    time::Duration,
};

use anyhow::Context;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app::{App, Screen},
    ui_render,
};

type Term = Terminal<CrosstermBackend<Stdout>>;

pub fn run() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let _guard = TerminalGuard;
    let mut app = App::new();
    loop {
        terminal.draw(|frame| ui_render::draw(frame, &app))?;
        if app.screen == Screen::Waiting
            && let Err(error) = app.poll_connection()
        {
            app.set_error(error);
        }
        app.tick();
        if event::poll(Duration::from_millis(250))? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Esc => {
                    if app.is_downloading() {
                        continue;
                    }
                    if app.screen == Screen::Files {
                        app.screen = Screen::MainMenu;
                        app.status = "Choose an action.".to_owned();
                    } else {
                        break;
                    }
                }
                KeyCode::Up => move_selection(&mut app, -1),
                KeyCode::Down => move_selection(&mut app, 1),
                KeyCode::Enter => activate(&mut app),
                _ => {}
            }
        }
    }
    Ok(())
}

fn setup_terminal() -> anyhow::Result<Term> {
    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("create terminal")
}

fn activate(app: &mut App) {
    if app.is_downloading() {
        return;
    }
    let result = match app.screen {
        Screen::Waiting => Ok(()),
        Screen::MainMenu => app.open_files(),
        Screen::Files => app.start_backup_selected(),
    };
    if let Err(error) = result {
        app.set_error(error);
    }
}

fn move_selection(app: &mut App, delta: isize) {
    if app.is_downloading() {
        return;
    }
    match app.screen {
        Screen::Waiting => {}
        Screen::MainMenu => {
            app.main_selected = 0;
        }
        Screen::Files => {
            let len = app.files.len() + 1;
            app.file_selected = wrap_index(app.file_selected, len, delta);
        }
    }
}

fn wrap_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    let len_signed = len as isize;
    (current as isize + delta).rem_euclid(len_signed) as usize
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
