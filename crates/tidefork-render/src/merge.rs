use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use primitive_app::{App, Change};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Input, Rect, Sheet, SheetAction, TextStyle, BLACK,
    BODY, CAPTION, PAPER,
};
use tidefork_core::{merge::MergeGame, Direction, SIZE};

use crate::{arrow, centered_icon, PuzzleView};

const BOARD_TOP: i32 = 365;
const SIDE: i32 = 235;

pub struct View {
    game: MergeGame,
    path: Option<PathBuf>,
    puzzles: PuzzleView,
    in_puzzles: bool,
    confirming: bool,
    needs_save: bool,
}

impl View {
    pub fn open(directory: PathBuf) -> Self {
        let path = directory.join("2048.json");
        let game = read_game(&path).unwrap_or_else(|| {
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            MergeGame::new(seed)
        });
        Self {
            game,
            path: Some(path),
            puzzles: PuzzleView::open(directory.join("tidefork.json")),
            in_puzzles: false,
            confirming: false,
            needs_save: false,
        }
    }

    pub fn preview() -> Self {
        Self {
            game: MergeGame::new(2048),
            path: None,
            puzzles: PuzzleView::preview(),
            in_puzzles: false,
            confirming: false,
            needs_save: false,
        }
    }

    fn board_changed(&mut self) -> Change {
        self.needs_save = true;
        let _ = self.save_board();
        Change::Partial
    }

    fn save_board(&mut self) -> io::Result<()> {
        if self.needs_save {
            if let Some(path) = &self.path {
                primitive_app::save(path, &serde_json::to_vec(&self.game)?)?;
            }
            self.needs_save = false;
        }
        Ok(())
    }

    fn slide(&mut self, direction: Direction) -> Change {
        if self.game.play(direction) {
            self.board_changed()
        } else {
            Change::None
        }
    }

    fn tap(&mut self, x: f32, y: f32) -> Change {
        let width = primitive_app::WIDTH as i32;
        if self.confirming {
            match restart_sheet(screen_rect(width, 0, 0, 1264, 1680)).hit(x, y) {
                Some(SheetAction::Confirm) => {
                    self.game.restart();
                    self.board_changed();
                }
                Some(SheetAction::Cancel) => {}
                None => return Change::None,
            }
            self.confirming = false;
            return Change::Full;
        }
        if back_button(width).contains(x, y) {
            return Change::Exit;
        }
        if puzzles_button(width).contains(x, y) {
            self.in_puzzles = true;
            return Change::Full;
        }
        if new_button(width).contains(x, y) {
            if self.game.finished() {
                self.game.restart();
                return self.board_changed();
            }
            self.confirming = true;
            return Change::Full;
        }
        for direction in Direction::ALL {
            if direction_button(width, direction).contains(x, y) {
                return self.slide(direction);
            }
        }
        Change::None
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        if self.in_puzzles {
            self.puzzles.draw(canvas);
            return;
        }

        let w = canvas.bounds().w;
        header(canvas, "2048", "Exit");
        draw_score(canvas, 40, "Score", self.game.score());
        draw_score(canvas, 648, "Best", self.game.best());
        let finished = self.game.finished();
        let message = if self.needs_save {
            "Could not save the board."
        } else if finished {
            "No moves left. Start a new game."
        } else {
            "Swipe to merge tiles and score."
        };
        canvas.text(screen_rect(w, 40, 292, 1184, 58), message, BODY.center());

        canvas.panel(screen_rect(w, 145, BOARD_TOP, 974, 974), 24, 3, PAPER);
        for (i, &value) in self.game.board().iter().enumerate() {
            let x = 162 + (i % SIZE) as i32 * SIDE;
            let y = BOARD_TOP + 17 + (i / SIZE) as i32 * SIDE;
            let (border, fill, ink) = match value {
                0 => (1, 247, BLACK),
                2..=8 => (3, PAPER, BLACK),
                16..=128 => (7, PAPER, BLACK),
                _ => (3, BLACK, PAPER),
            };
            canvas.panel(screen_rect(w, x + 9, y + 9, 217, 217), 18, border, fill);
            if value > 0 {
                let size = match value {
                    2..=999 => 81,
                    1000..=9999 => 63,
                    _ => 37,
                };
                canvas.text(
                    screen_rect(w, x + 16, y + 63, 203, 118),
                    &value.to_string(),
                    TextStyle::new(size).bold().center().ink(ink),
                );
            }
        }

        for direction in Direction::ALL {
            let bounds = direction_button(w, direction);
            let pressed = canvas.control_pressed(bounds, false);
            let (fill, ink) = if pressed {
                (BLACK, PAPER)
            } else {
                (PAPER, BLACK)
            };
            canvas.panel(bounds, 16, 4, fill);
            canvas.icon(centered_icon(bounds, canvas.px(46)), arrow(direction), ink);
        }
        canvas.button(puzzles_button(w), "Line puzzles", false, false);
        canvas.button(new_button(w), "New game", finished, false);

        if self.confirming {
            restart_sheet(canvas.bounds()).draw(canvas, None);
        }
    }

    fn input(&mut self, input: Input) -> Change {
        if self.in_puzzles {
            return match self.puzzles.input(input) {
                Change::Exit => {
                    self.in_puzzles = false;
                    Change::Full
                }
                change => change,
            };
        }

        match input {
            Input::Tap(x, y) => self.tap(x, y),
            Input::Swipe { dx, dy, start_y }
                if !self.confirming
                    && (BOARD_TOP as f32..(BOARD_TOP + 974) as f32).contains(&start_y)
                    && dx.abs().max(dy.abs()) >= 36.0 =>
            {
                let direction = if dx.abs() >= dy.abs() {
                    if dx > 0.0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    }
                } else if dy > 0.0 {
                    Direction::Down
                } else {
                    Direction::Up
                };
                self.slide(direction)
            }
            Input::PageBack if self.confirming => {
                self.confirming = false;
                Change::Full
            }
            Input::PageBack => Change::Exit,
            _ => Change::None,
        }
    }

    fn save(&mut self) -> io::Result<()> {
        self.save_board()?;
        self.puzzles.save()
    }
}

fn read_game(path: &std::path::Path) -> Option<MergeGame> {
    let mut bytes = Vec::new();
    File::open(path)
        .ok()?
        .take(4097)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 4096 {
        return None;
    }
    MergeGame::resume(serde_json::from_slice(&bytes).ok()?)
}

fn draw_score(canvas: &mut Canvas, x: i32, name: &str, score: u64) {
    let w = canvas.bounds().w;
    canvas.card(screen_rect(w, x, 164, 576, 115));
    canvas.text(screen_rect(w, x + 28, 172, 244, 98), name, CAPTION);
    canvas.text(
        screen_rect(w, x + 178, 172, 370, 98),
        &score.to_string(),
        TextStyle::new(45).bold().right(),
    );
}

fn direction_button(width: i32, direction: Direction) -> Rect {
    let position = match direction {
        Direction::Left => 0,
        Direction::Up => 1,
        Direction::Down => 2,
        Direction::Right => 3,
    };
    screen_rect(width, 162 + position * 242, 1370, 214, 100)
}

fn puzzles_button(width: i32) -> Rect {
    screen_rect(width, 40, 1531, 567, 112)
}

fn new_button(width: i32) -> Rect {
    screen_rect(width, 655, 1531, 569, 112)
}

fn restart_sheet(bounds: Rect) -> Sheet<'static> {
    Sheet::centered(
        bounds,
        "Start a new game?",
        "This clears the current board and keeps your best score.",
        "New game",
    )
}
