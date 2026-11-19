//! A scored 4×4 sliding board. The line puzzles use their own move rules and save file.

use serde::{Deserialize, Serialize};

use crate::{Direction, SIZE};

pub const GOAL: u32 = 2048;
type Board = [u32; SIZE * SIZE];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeGame {
    board: Board,
    score: u64,
    best: u64,
    random: u64,
}

impl MergeGame {
    pub fn new(seed: u64) -> Self {
        let mut game = Self {
            board: [0; SIZE * SIZE],
            score: 0,
            best: 0,
            random: seed,
        };
        game.spawn();
        game.spawn();
        game
    }

    pub fn resume(saved: Self) -> Option<Self> {
        let occupied = saved.board.iter().filter(|&&value| value != 0).count();
        let valid = saved
            .board
            .iter()
            .all(|&value| value == 0 || (value >= 2 && value.is_power_of_two()));
        (occupied >= 2 && valid && saved.best >= saved.score).then_some(saved)
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn best(&self) -> u64 {
        self.best
    }

    pub fn reached_goal(&self) -> bool {
        self.board.iter().any(|&value| value >= GOAL)
    }

    pub fn finished(&self) -> bool {
        Direction::ALL
            .into_iter()
            .all(|direction| slide(self.board, direction).0 == self.board)
    }

    pub fn play(&mut self, direction: Direction) -> bool {
        let (board, gained) = slide(self.board, direction);
        if board == self.board {
            return false;
        }

        self.board = board;
        self.score = self.score.saturating_add(gained);
        self.best = self.best.max(self.score);
        self.spawn();
        true
    }

    pub fn restart(&mut self) {
        let best = self.best;
        *self = Self::new(self.next_random());
        self.best = best;
    }

    fn spawn(&mut self) {
        let count = self.board.iter().filter(|&&value| value == 0).count();
        if count == 0 {
            return;
        }

        let position = (self.next_random() % count as u64) as usize;
        let value = if self.next_random().is_multiple_of(10) {
            4
        } else {
            2
        };
        if let Some(tile) = self
            .board
            .iter_mut()
            .filter(|tile| **tile == 0)
            .nth(position)
        {
            *tile = value;
        }
    }

    fn next_random(&mut self) -> u64 {
        // Keep the state in the save so resuming doesn't reroll the next move.
        self.random = self.random.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.random;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }
}

fn slide(board: Board, direction: Direction) -> (Board, u64) {
    let mut result = [0; SIZE * SIZE];
    let mut score = 0;

    for lane in 0..SIZE {
        let cells = std::array::from_fn::<_, SIZE, _>(|step| match direction {
            Direction::Left => lane * SIZE + step,
            Direction::Right => lane * SIZE + (SIZE - 1 - step),
            Direction::Up => step * SIZE + lane,
            Direction::Down => (SIZE - 1 - step) * SIZE + lane,
        });
        let mut tiles = cells
            .iter()
            .map(|&cell| board[cell])
            .filter(|&value| value != 0)
            .peekable();
        for cell in cells {
            let Some(value) = tiles.next() else {
                break;
            };
            let merged = value
                .checked_mul(2)
                .filter(|_| tiles.peek() == Some(&value));
            result[cell] = match merged {
                Some(value) => {
                    tiles.next();
                    score += u64::from(value);
                    value
                }
                None => value,
            };
        }
    }

    (result, score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slides_every_lane_and_merges_each_tile_once() {
        let board = [2, 2, 2, 2, 4, 4, 8, 0, 0, 0, 0, 0, 0, 2, 0, 2];
        assert_eq!(
            slide(board, Direction::Left),
            ([4, 4, 0, 0, 8, 8, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0], 20)
        );
        assert_eq!(&slide(board, Direction::Right).0[..4], &[0, 0, 4, 4]);
        let vertical = [0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 4, 0, 0];
        assert_eq!(
            slide(vertical, Direction::Up).0,
            [0, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            slide(vertical, Direction::Down).0,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0]
        );
    }

    #[test]
    fn only_a_changed_board_spawns_and_reaching_goal_preserves_play() {
        let mut game = MergeGame::new(7);
        assert_eq!(game.board.iter().filter(|&&value| value != 0).count(), 2);
        game.board = [1024, 0, 0, 0, 1024, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let before = game.clone();
        assert!(!game.play(Direction::Left));
        assert_eq!(game, before);
        assert!(game.play(Direction::Up));
        assert_eq!(game.board[0], GOAL);
        assert_eq!(game.board.iter().filter(|&&value| value != 0).count(), 2);
        assert!(game.reached_goal());
        assert!(!game.finished());
        assert_eq!(game.score, 2048);
        game.restart();
        assert_eq!(game.score, 0);
        assert_eq!(game.best, 2048);
    }

    #[test]
    fn recognizes_a_full_board_with_no_merges_and_rejects_invalid_saves() {
        let mut game = MergeGame::new(11);
        game.board = [2, 4, 2, 4, 4, 2, 4, 2, 2, 4, 2, 4, 4, 2, 4, 2];
        assert!(game.finished());
        assert_eq!(MergeGame::resume(game.clone()), Some(game.clone()));
        game.board[0] = 3;
        assert_eq!(MergeGame::resume(game), None);
    }
}
