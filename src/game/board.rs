use std::fmt::{Display, Formatter};
use std::ops::{AddAssign, Div, Index, IndexMut, Rem, Sub};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cell {
    Died,
    Dead,
    Born,
    Alive,
}

impl Cell {
    pub fn is_alive(&self) -> bool {
        match self {
            Cell::Died => false,
            Cell::Dead => false,
            Cell::Born => true,
            Cell::Alive => true,
        }
    }

    pub fn flip(&mut self) {
        if self.is_alive() {
            *self = Cell::Died
        } else {
            *self = Cell::Born
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Dead => write!(f, "X"),
            Cell::Alive => write!(f, "O"),
            Cell::Died => write!(f, "x"),
            Cell::Born => write!(f, "o"),
        }
    }
}

impl From<Cell> for char {
    fn from(cell: Cell) -> Self {
        match cell {
            Cell::Died => 'x',
            Cell::Dead => 'X',
            Cell::Born => 'o',
            Cell::Alive => 'O',
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    inner: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Board {
    pub fn check_index(&self, (x, y): (usize, usize)) -> bool {
        x < self.width() && y < self.height()
    }
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        if width == 0 || height == 0 {
            panic!("board cannot be zero sized")
        }
        Board {
            inner: vec![Cell::Dead; width * height],
            width,
            height,
        }
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn iter(&self) -> BoardIter { self.into_iter() }

    pub fn set(&mut self, (x, y): (usize, usize), cell: Cell) {
        assert!(x < self.width, "x index {} is out of bound in width {}", x, self.width);
        assert!(y < self.height, "y index {} is out of bound in height {}", y, self.height);
        self.inner[y * self.width + x] = cell
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for row in 0..self.height() {
            for col in 0..self.width() {
                str.push(self[(col, row)].into());
            }
            str.push('\n');
        }
        write!(f, "Board: \n{}", str)
    }
}

impl Index<(usize, usize)> for Board {
    type Output = Cell;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        assert!(x < self.width);
        assert!(y < self.height);
        &self.inner[y * self.width + x]
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        assert!(x < self.width, "x index {} is out of bound in width {}", x, self.width);
        assert!(y < self.height, "y index {} is out of bound in height {}", y, self.height);
        &mut self.inner[y * self.width + x]
    }
}

pub struct BoardIter<'a> {
    board: &'a Board,
    index: Option<(usize, usize)>,
    width: usize,
    height: usize,
}

impl<'a> BoardIter<'a> {
    fn new(board: &'a Board) -> Self {
        BoardIter {
            board,
            index: Some((0, 0)),
            width: board.width(),
            height: board.height(),
        }
    }

    fn increment_index(&mut self) {
        if let Some((x, y)) = self.index {
            let mut x = x;
            let mut y = y + 1;
            if y >= self.height {
                x += 1;
                y = 0;
            }
            if x >= self.width || y >= self.height {
                self.index = None;
            } else {
                self.index = Some((x, y));
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Entry {
    cell: Cell,
    index: (usize, usize),
}

impl Entry {
    pub fn new(cell: Cell, index: (usize, usize)) -> Self {
        Entry {
            cell,
            index,
        }
    }

    pub fn cell(&self) -> Cell { self.cell }
    pub fn index(&self) -> (usize, usize) { self.index }
    pub fn x(&self) -> usize { self.index.0 }
    pub fn y(&self) -> usize { self.index.1 }
}

impl<'a> Iterator for BoardIter<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.index.map(|index| {
            self.increment_index();
            let cell = self.board[index];
            Entry::new(cell, index)
        })
    }
}

impl<'a> IntoIterator for &'a Board {
    type Item = Entry;
    type IntoIter = BoardIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BoardIter::new(self)
    }
}


#[cfg(test)]
mod tests {
    use crate::game::board::{Board, Cell};

    #[test]
    fn create_board() {
        let board = Board::new(5, 8);
        assert_eq!(board.width(), 5);
        assert_eq!(board.height(), 8);
    }

    #[test]
    #[should_panic]
    fn create_not_valid_board() {
        let _board = Board::new(0, 0);
    }

    #[test]
    fn board_index() {
        let mut board = Board::new(2, 2);
        board[(1, 1)] = Cell::Alive;
        assert_eq!(Cell::Alive, board[(1, 1)]);
        assert_eq!(Cell::Dead, board[(1, 0)]);
    }

    #[test]
    #[should_panic]
    fn board_index_not_valid() {
        let mut board = Board::new(2, 2);
        board[(1, 3)] = Cell::Alive;
    }

    #[test]
    fn iterator() {
        let mut board = Board::new(2, 2);
        board[(0, 0)] = Cell::Alive;
        board[(0, 1)] = Cell::Alive;
        let str = board.iter().fold(String::new(), |mut acc, entry| {
            acc.push_str(entry.cell().to_string().as_str());
            acc
        });
        assert_eq!("OOXX", str);
    }

    #[test]
    fn cell_flip() {
        let mut cell = Cell::Alive;
        cell.flip();
        assert_eq!(cell, Cell::Died);
        cell.flip();
        assert_eq!(cell, Cell::Born);
    }
}
