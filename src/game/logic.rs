use std::ops::{Not, Rem};
use crate::game::board::{Board, Cell};


pub fn next_state(board: &mut Board) -> bool {
    let snapshot = board.clone();
    for entry in snapshot.iter() {
        let cell = entry.cell();
        let live_neighbours = count_live_neighbours(&snapshot, entry.index());
        let new_cell = match cell {
            Cell::Dead if live_neighbours == 3 => Cell::Live,
            Cell::Live if live_neighbours < 2 => Cell::Dead,
            Cell::Live if live_neighbours > 3 => Cell::Dead,
            _ => cell,
        };
        board[entry.index()] = new_cell;
    }
    *board != snapshot
}


fn count_live_neighbours(board: &Board, (ux, uy): (usize, usize)) -> u8 {
    let mut live_neighbours = 0;
    let x = ux as isize;
    let y = uy as isize;
    for x in (x - 1)..=(x + 1) {
        for y in (y - 1)..=(y + 1) {
            let _ = valid_neighbour_index(board, (ux, uy), x, y)
                .filter(|&index| board[index] == Cell::Live)
                .map(|_| live_neighbours += 1);
        }
    }
    live_neighbours
}

fn valid_neighbour_index(board: &Board, (ux, uy): (usize, usize), x: isize, y: isize) -> Option<(usize, usize)> {
    if x == ux as isize && y == uy as isize {
        return None;
    }
    let x = x.rem_euclid(board.width() as isize) as usize;
    let y = y.rem_euclid(board.height() as isize) as usize;
    Some((x, y))
}


#[cfg(test)]
mod tests {
    use std::ops::Rem;

    #[test]
    fn rem_check() {
        let x = 12;
        let x = x.rem(10);
        assert_eq!(x, 2);

        let x = 3;
        let x = x.rem(10);
        assert_eq!(x, 3);

        let x = -3i32;
        let x = x.rem_euclid(10);
        assert_eq!(x, 10-3);
    }
}