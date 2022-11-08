//The universe of the Game of Life is an infinite, two-dimensional orthogonal grid of square cells, each of which is in one of two possible states, live or dead (or populated and unpopulated, respectively). Every cell interacts with its eight neighbours, which are the cells that are horizontally, vertically, or diagonally adjacent. At each step in time, the following transitions occur:
//
// Any live cell with fewer than two live neighbours dies, as if by underpopulation.
// Any live cell with two or three live neighbours lives on to the next generation.
// Any live cell with more than three live neighbours dies, as if by overpopulation.
// Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
// These rules, which compare the behavior of the automaton to real life, can be condensed into the following:
//
// Any live cell with two or three live neighbours survives.
// Any dead cell with three live neighbours becomes a live cell.
// All other live cells die in the next generation. Similarly, all other dead cells stay dead.
// The initial pattern constitutes the seed of the system. The first generation is created by applying the above rules simultaneously to every cell in the seed, live or dead; births and deaths occur simultaneously, and the discrete moment at which this happens is sometimes called a tick.[nb 1] Each generation is a pure function of the preceding one. The rules continue to be applied repeatedly to create further generations.

use std::cmp::min;
use std::io::stdout;
use std::ops::IndexMut;
use std::time::{Duration, Instant};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::ExecutableCommand;
use crossterm::terminal::{Clear, ClearType};

use crate::game::board::{Board, Cell};
use crate::game::logic::{next_state, resize};
use crate::game::tui::{BoardEvent, DEFAULT_THEME, draw_board, get_size, handle_events};

mod game;


fn main() {
    crossterm::terminal::enable_raw_mode().unwrap();
    let mut board = {
        let (w, h) = get_size().unwrap();
        Board::new(w as usize, h as usize)
    };

    for i in 0..min(25usize, min(board.width(), board.height())) {
        board.index_mut((i, i)).flip();
    }

    let mut stdout = stdout();
    stdout.execute(EnableMouseCapture).unwrap();

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
        draw_board(&DEFAULT_THEME, &mut stdout, &board).unwrap();

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
    stdout.execute(DisableMouseCapture).unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum PauseState {
    Disabled,
    JustEnabled,
    Activated,
}


