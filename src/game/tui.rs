use std::io::{Stdout, Write};
use std::time::Duration;

use crossterm::{cursor, QueueableCommand, Result, style};
use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind, poll, read};
use crossterm::style::{ContentStyle, StyledContent, Stylize};
use crossterm::terminal;
use once_cell::sync::Lazy;

use crate::{Board, Cell};

#[derive(Debug)]
pub struct Theme {
    dead_cell_style: ContentStyle,
    alive_cell_style: ContentStyle,
    died_cell_style: ContentStyle,
    born_cell_style: ContentStyle,
    dead_cell_content: String,
    alive_cell_content: String,
    died_cell_content: String,
    born_cell_content: String,
}

impl Theme {
    pub fn new(
        dead_cell_style: ContentStyle,
        alive_cell_style: ContentStyle,
        died_cell_style: ContentStyle,
        born_cell_style: ContentStyle,
        dead_cell_content: String,
        alive_cell_content: String,
        died_cell_content: String,
        born_cell_content: String,
    ) -> Self {
        Self {
            dead_cell_style,
            alive_cell_style,
            died_cell_style,
            born_cell_style,
            dead_cell_content,
            alive_cell_content,
            died_cell_content,
            born_cell_content,
        }
    }
}

pub static DEFAULT_THEME: Lazy<Theme> = Lazy::new(|| {
    Theme::new(
        ContentStyle::new().white(),
        ContentStyle::new().yellow(),
        ContentStyle::new().red(),
        ContentStyle::new().green(),
        "█".to_string(),
        "█".to_string(),
        "█".to_string(),
        "█".to_string(),
    )
});


pub fn draw_board(
    theme: &Theme,
    stdout: &mut Stdout,
    board: &Board,
) -> Result<()> {
    let dead_style = StyledContent::new(
        theme.dead_cell_style,
        theme.dead_cell_content.as_str(),
    );
    let alive_style = StyledContent::new(
        theme.alive_cell_style,
        theme.alive_cell_content.as_str(),
    );
    let died_style = StyledContent::new(
        theme.died_cell_style,
        theme.died_cell_content.as_str(),
    );
    let born_style = StyledContent::new(
        theme.born_cell_style,
        theme.born_cell_content.as_str(),
    );
    for entry in board.iter() {
        let style = match entry.cell() {
            Cell::Dead => dead_style,
            Cell::Alive => alive_style,
            Cell::Died => died_style,
            Cell::Born => born_style,
        };
        stdout
            .queue(cursor::MoveTo(entry.x() as u16, entry.y() as u16))?
            .queue(style::PrintStyledContent(style))?;
    }
    stdout.flush()?;
    Ok(())
}

#[derive(Debug)]
pub enum BoardEvent {
    MouseClick {
        x: u16,
        y: u16,
    },
    Exit,
    Resized {
        x: u16,
        y: u16,
    },
    Pause,
    Speed(bool),
}

pub fn handle_events(
    poll_duration: Duration,
) -> Option<BoardEvent> {
    if poll(poll_duration).ok()? {
        return match read().ok()? {
            Event::Mouse(
                MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left),
                    column: x,
                    row: y,
                    ..
                }) => {
                Some(BoardEvent::MouseClick { x, y })
            }
            Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                Some(BoardEvent::Exit)
            }
            Event::Key(KeyEvent { code: KeyCode::Char(' '), .. }) => {
                Some(BoardEvent::Pause)
            }
            Event::Key(KeyEvent { code: KeyCode::Char('+'), .. }) => {
                Some(BoardEvent::Speed(true))
            }
            Event::Key(KeyEvent { code: KeyCode::Char('-'), .. }) => {
                Some(BoardEvent::Speed(false))
            }
            Event::Resize(x, y) => {
                Some(BoardEvent::Resized { x, y })
            }
            _ => None
        };
    } else {
        None
    }
}

pub fn get_size() -> Result<(u16, u16)> {
    terminal::enable_raw_mode().unwrap();
    terminal::size()
}

