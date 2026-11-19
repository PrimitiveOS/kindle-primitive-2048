//! Tidefork slides one line at a time. Each tile can merge once per move.

use serde::{Deserialize, Serialize};

pub const SIZE: usize = 4;
pub type Board = [u16; SIZE * SIZE];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Up, Self::Down];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Move {
    pub direction: Direction,
    pub lane: usize,
}

pub struct Puzzle {
    pub target: u16,
    pub limit: usize,
    start: Board,
}

pub const PUZZLES: &[Puzzle] = &[
    Puzzle {
        target: 16,
        limit: 2,
        start: [2, 0, 8, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0],
    },
    Puzzle {
        target: 16,
        limit: 3,
        start: [0, 0, 0, 0, 4, 2, 0, 2, 2, 0, 0, 0, 4, 8, 0, 0],
    },
    Puzzle {
        target: 32,
        limit: 3,
        start: [4, 4, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 16],
    },
    Puzzle {
        target: 32,
        limit: 4,
        start: [0, 4, 2, 0, 0, 8, 0, 0, 0, 0, 8, 0, 4, 16, 2, 0],
    },
    Puzzle {
        target: 64,
        limit: 4,
        start: [0, 4, 8, 0, 0, 0, 0, 2, 0, 0, 0, 2, 0, 8, 16, 32],
    },
    Puzzle {
        target: 64,
        limit: 4,
        start: [8, 0, 0, 16, 0, 2, 8, 0, 4, 0, 0, 0, 0, 0, 4, 32],
    },
    Puzzle {
        target: 64,
        limit: 5,
        start: [16, 0, 0, 0, 4, 4, 8, 8, 2, 0, 8, 0, 0, 0, 0, 16],
    },
    Puzzle {
        target: 64,
        limit: 5,
        start: [0, 0, 16, 4, 8, 2, 0, 16, 0, 4, 0, 0, 8, 0, 16, 0],
    },
    Puzzle {
        target: 128,
        limit: 5,
        start: [4, 0, 32, 0, 32, 16, 8, 0, 8, 0, 0, 0, 0, 32, 16, 0],
    },
    Puzzle {
        target: 128,
        limit: 6,
        start: [8, 16, 0, 32, 8, 0, 16, 0, 32, 0, 0, 0, 4, 16, 4, 0],
    },
];

fn slide(board: &mut Board, turn: Move) -> bool {
    if turn.lane >= SIZE {
        return false;
    }
    let indices: [usize; SIZE] = std::array::from_fn(|i| match turn.direction {
        Direction::Left => turn.lane * SIZE + i,
        Direction::Right => turn.lane * SIZE + (SIZE - 1 - i),
        Direction::Up => i * SIZE + turn.lane,
        Direction::Down => (SIZE - 1 - i) * SIZE + turn.lane,
    });
    let previous = indices.map(|i| board[i]);
    let mut nonzero = previous.into_iter().filter(|&value| value != 0).peekable();
    let merged: [u16; SIZE] = std::array::from_fn(|_| {
        let Some(tile) = nonzero.next() else { return 0 };
        if nonzero.peek() == Some(&tile) {
            nonzero.next();
            tile * 2
        } else {
            tile
        }
    });
    for (index, value) in indices.into_iter().zip(merged) {
        board[index] = value;
    }
    previous != merged
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Playing,
    Complete,
    OutOfMoves,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Progress {
    level: usize,
    moves: Vec<Move>,
}

pub struct Game {
    progress: Progress,
    board: Board,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            progress: Progress::default(),
            board: PUZZLES[0].start,
        }
    }

    /// Start from the canonical board and accept only a valid, bounded move sequence.
    pub fn resume(progress: Progress) -> Option<Self> {
        let puzzle = PUZZLES.get(progress.level)?;
        if progress.moves.len() > puzzle.limit {
            return None;
        }
        let mut game = Self {
            progress: Progress {
                level: progress.level,
                moves: Vec::new(),
            },
            board: puzzle.start,
        };
        for turn in progress.moves {
            if !game.play(turn) {
                return None;
            }
        }
        Some(game)
    }

    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    pub fn level(&self) -> usize {
        self.progress.level
    }

    pub fn puzzle(&self) -> &'static Puzzle {
        &PUZZLES[self.level()]
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn moves_left(&self) -> usize {
        self.puzzle().limit - self.progress.moves.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.progress.moves.is_empty()
    }

    pub fn status(&self) -> Status {
        if self
            .board
            .iter()
            .any(|&value| value >= self.puzzle().target)
        {
            Status::Complete
        } else if self.moves_left() == 0 {
            Status::OutOfMoves
        } else {
            Status::Playing
        }
    }

    pub fn play(&mut self, turn: Move) -> bool {
        if self.status() != Status::Playing || !slide(&mut self.board, turn) {
            return false;
        }
        self.progress.moves.push(turn);
        true
    }

    pub fn undo(&mut self) -> bool {
        if self.progress.moves.pop().is_none() {
            return false;
        }
        self.board = self.puzzle().start;
        for &turn in &self.progress.moves {
            slide(&mut self.board, turn);
        }
        true
    }

    pub fn reset(&mut self) {
        self.board = self.puzzle().start;
        self.progress.moves.clear();
    }

    pub fn advance(&mut self) -> bool {
        if self.status() != Status::Complete {
            return false;
        }
        self.progress.level = (self.level() + 1) % PUZZLES.len();
        self.reset();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(action: usize) -> Move {
        let direction = match (action >= 8, action % 2 == 1) {
            (false, false) => Direction::Left,
            (false, true) => Direction::Right,
            (true, false) => Direction::Up,
            (true, true) => Direction::Down,
        };
        Move {
            direction,
            lane: action / 2 % SIZE,
        }
    }

    #[test]
    fn each_tile_merges_once_and_only_its_line_changes() {
        let mut board = [0, 0, 0, 8, 2, 2, 2, 2, 0, 4, 8, 0, 0, 0, 0, 0];
        assert!(slide(&mut board, turn(2)));
        assert_eq!(board, [0, 0, 0, 8, 4, 4, 0, 0, 0, 4, 8, 0, 0, 0, 0, 0]);
        assert!(slide(&mut board, turn(3)));
        assert_eq!(&board[4..8], &[0, 0, 0, 8]);
    }

    #[test]
    fn all_ten_puzzles_can_finish_in_their_move_budget() {
        let solutions: &[&[usize]] = &[
            &[0, 10],
            &[5, 9, 6],
            &[0, 1, 14],
            &[12, 2, 10, 2],
            &[0, 11, 6, 6],
            &[1, 12, 1, 14],
            &[3, 12, 0, 14, 0],
            &[2, 8, 1, 12, 12],
            &[3, 10, 12, 12, 0],
            &[7, 8, 12, 0, 8, 0],
        ];
        assert_eq!(solutions.len(), PUZZLES.len());
        let mut game = Game::new();
        for (index, solution) in solutions.iter().enumerate() {
            assert_eq!(game.level(), index);
            for &action in *solution {
                assert!(game.play(turn(action)), "puzzle {}", index + 1);
            }
            assert_eq!(game.status(), Status::Complete);
            assert!(!game.play(turn(0)));
            assert!(game.advance());
        }
        assert_eq!(game.level(), 0);
    }

    #[test]
    fn ineffective_moves_do_not_spend_turns_and_saved_play_can_be_undone() {
        let mut game = Game::new();
        assert!(!game.play(turn(4)));
        assert!(!game.play(Move {
            lane: SIZE,
            direction: Direction::Up
        }));
        assert_eq!(game.moves_left(), 2);
        assert!(game.play(turn(0)));
        let mut reopened = Game::resume(game.progress().clone()).unwrap();
        assert_eq!(reopened.board(), game.board());
        assert!(reopened.undo());
        assert_eq!(reopened.board(), Game::new().board());
        assert_eq!(reopened.status(), Status::Playing);
        assert!(Game::resume(Progress {
            level: PUZZLES.len(),
            moves: vec![]
        })
        .is_none());
        assert!(Game::resume(Progress {
            level: 0,
            moves: vec![turn(4)]
        })
        .is_none());
    }
}
