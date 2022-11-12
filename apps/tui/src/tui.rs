use std::cmp::min;
use std::io::{Stdout, Write};
use std::io::stdout;
use std::ops::IndexMut;
use std::time::{Duration, Instant};

use crossterm::{cursor, QueueableCommand, Result, style};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind, poll, read};
use crossterm::ExecutableCommand;
use crossterm::style::{ContentStyle, StyledContent, Stylize};
use crossterm::terminal::{Clear, ClearType};
use crossterm::terminal;
use once_cell::sync::Lazy;

use game_of_life::board::{Board, Cell};
use game_of_life::logic::{next_state, resize};

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

pub fn main_loop() -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut board = {
        let (w, h) = get_size()?;
        Board::new(w as usize, h as usize)
    };

    for i in 0..min(25usize, min(board.width(), board.height())) {
        board.index_mut((i, i)).flip();
    }

    let mut stdout = stdout();
    stdout.execute(EnableMouseCapture)?;

    let mut frame_duration = Duration::from_millis(64);

    fn remaining_time(start: Instant, frame_duration: Duration) -> Option<Duration> {
        let now = Instant::now();
        let passed = now - start;
        if passed >= frame_duration {
            None
        } else {
            Some(frame_duration - passed)
        }
    }

    let mut pause_state = PauseState::Disabled;
    let mut last_updated = Instant::now();

    'outer: loop {
        let start = Instant::now();
        let should_compute_state = Instant::now() > last_updated + frame_duration;
        draw_board(&DEFAULT_THEME, &mut stdout, &board)?;

        while let Some(timeout) = remaining_time(start, Duration::from_millis(16)) {
            if let Some(event) = handle_events(timeout) {
                match event {
                    BoardEvent::MouseClick { x, y } => {
                        let x = x as usize;
                        let y = y as usize;
                        if board.check_index((x, y)) {
                            board.index_mut((x, y)).flip();
                        }
                    }
                    BoardEvent::Exit => {
                        break 'outer;
                    }
                    BoardEvent::Resized { x, y } => {
                        resize(&mut board, x as usize, y as usize);
                    }
                    BoardEvent::Pause => {
                        if pause_state != PauseState::Disabled {
                            pause_state = PauseState::Disabled;
                        } else {
                            pause_state = PauseState::JustEnabled;
                        }
                    }
                    BoardEvent::Speed(increase) => {
                        if increase {
                            frame_duration /= 2;
                        } else {
                            frame_duration *= 2;
                        }
                    }
                }
            }
        }


        let is_paused = match pause_state {
            PauseState::Disabled => false,
            PauseState::JustEnabled => {
                pause_state = PauseState::Activated;
                false
            }
            _ => true,
        };
        if should_compute_state && !is_paused {
            next_state(&mut board);
            last_updated = Instant::now();
        }
    }
    stdout.execute(DisableMouseCapture)?;
    stdout.execute(Clear(ClearType::All))?;
    terminal::disable_raw_mode()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum PauseState {
    Disabled,
    JustEnabled,
    Activated,
}
