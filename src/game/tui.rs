use std::collections::HashMap;
use std::io::{Stdout, Write};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Duration;
use crossterm::{QueueableCommand, cursor, style, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind, poll, read};
use crossterm::style::{ContentStyle, StyledContent, Stylize};
use crossterm::terminal;
use once_cell::sync::Lazy;
use crate::{Board, Cell};
use crate::game::board::CellLifecycle;

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
        born_cell_content: String
    ) -> Self {
        Self {
            dead_cell_style,
            alive_cell_style,
            died_cell_style,
            born_cell_style,
            dead_cell_content,
            alive_cell_content,
            died_cell_content,
            born_cell_content
        }
    }
}

pub static DEFAULT_THEME: Lazy<Theme> = Lazy::new(|| {
    Theme {
        dead_cell_style: ContentStyle::new().black(),
        alive_cell_style: ContentStyle::new().yellow(),
        died_cell_style: ContentStyle::new().dark_red(),
        born_cell_style: ContentStyle::new().green(),
        dead_cell_content: "█".to_string(),
        alive_cell_content: "█".to_string(),
        died_cell_content: "█".to_string(),
        born_cell_content: "█".to_string()
    }
});


pub fn draw_board(
    theme: &Theme,
    stdout: &mut Stdout,
    board: &Board,
    lifecycle: &HashMap<(usize, usize), CellLifecycle>,
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
        let mut style = match entry.cell() {
            Cell::Dead => dead_style.clone(),
            Cell::Live => alive_style.clone(),
        };
        if let Some(&cell_lifecycle) = lifecycle.get(&entry.index()) {
            style = match cell_lifecycle {
                CellLifecycle::Died => died_style.clone(),
                CellLifecycle::Born => born_style.clone(),
            }
        }
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
    event_sink: Sender<BoardEvent>,
    should_exit: Receiver<()>,
) -> Result<()> {
    loop {
        match should_exit.try_recv() {
            Ok(_) => return Ok(()),
            Err(err) => match err {
                TryRecvError::Empty => (),
                TryRecvError::Disconnected => return Ok(())
            }
        }

        if poll(poll_duration)? {
            match read()? {
                Event::Mouse(
                    MouseEvent {
                        kind: MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left),
                        column: x,
                        row: y,
                        ..
                    }) => {
                    if let Err(err) = event_sink.send(BoardEvent::MouseClick { x, y }) {
                        let event = err.0;
                        eprintln!("Failed to send event: {:?}", event);
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                    if let Err(err) = event_sink.send(BoardEvent::Exit) {
                        let event = err.0;
                        eprintln!("Failed to send event: {:?}", event);
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Char(' '), .. }) => {
                    if let Err(err) = event_sink.send(BoardEvent::Pause) {
                        let event = err.0;
                        eprintln!("Failed to send event: {:?}", event);
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Char('+'), .. }) => {
                    if let Err(err) = event_sink.send(BoardEvent::Speed(true)) {
                        let event = err.0;
                        eprintln!("Failed to send event: {:?}", event);
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Char('-'), .. }) => {
                    if let Err(err) = event_sink.send(BoardEvent::Speed(false)) {
                        let event = err.0;
                        eprintln!("Failed to send event: {:?}", event);
                    }
                }
                Event::Resize(x, y) => {
                    if let Err(err) = event_sink.send(BoardEvent::Resized { x, y }) {
                        let event = err.0;
                        eprintln!("Failed to send event: {:?}", event);
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn get_size() -> Result<(u16, u16)> {
    terminal::enable_raw_mode().unwrap();
    terminal::size()
}

