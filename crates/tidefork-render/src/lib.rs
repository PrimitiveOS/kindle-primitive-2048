use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use primitive_app::{App, Change};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Icon, Input, Rect, Sheet, SheetAction, TextStyle,
    BLACK, BODY, CAPTION, PAPER,
};
use tidefork_core::{Direction, Game, Move, Status, PUZZLES, SIZE};

pub struct View {
    pub game: Game,
    path: Option<PathBuf>,
    confirming_reset: bool,
    notice: Option<&'static str>,
}

impl View {
    pub fn open(path: PathBuf) -> Self {
        let game = File::open(&path)
            .ok()
            .and_then(|file| {
                let mut bytes = Vec::new();
                file.take(4097).read_to_end(&mut bytes).ok()?;
                if bytes.len() > 4096 {
                    return None;
                }
                Game::resume(serde_json::from_slice(&bytes).ok()?)
            })
            .unwrap_or_default();
        Self {
            game,
            path: Some(path),
            confirming_reset: false,
            notice: None,
        }
    }

    pub fn preview() -> Self {
        Self {
            game: Game::new(),
            path: None,
            confirming_reset: false,
            notice: None,
        }
    }

    fn persist(&mut self) {
        if self.save().is_err() {
            self.notice = Some("Could not save this puzzle.");
        }
    }

    fn tap(&mut self, x: f32, y: f32) -> Change {
        let width = primitive_app::WIDTH as i32;
        if self.confirming_reset {
            match reset_sheet(screen_rect(width, 0, 0, 1264, 1680)).hit(x, y) {
                Some(SheetAction::Confirm) => {
                    self.game.reset();
                    self.notice = None;
                    self.persist();
                }
                Some(SheetAction::Cancel) => {}
                None => return Change::None,
            }
            self.confirming_reset = false;
            return Change::Full;
        }
        if back_button(width).contains(x, y) {
            return Change::Exit;
        }
        if undo_button(width).contains(x, y) && self.game.undo() {
            self.notice = None;
            self.persist();
            return Change::Partial;
        }
        if action_button(width).contains(x, y) {
            if self.game.advance() {
                self.notice = None;
                self.persist();
                return Change::Full;
            }
            if self.game.can_undo() {
                self.confirming_reset = true;
                return Change::Full;
            }
        }
        if self.game.status() != Status::Playing {
            return Change::None;
        }
        for direction in Direction::ALL {
            for lane in 0..SIZE {
                if arrow_button(width, direction, lane).contains(x, y) {
                    if self.game.play(Move { direction, lane }) {
                        self.notice = None;
                        self.persist();
                    } else {
                        self.notice = Some("That line cannot move this way.");
                    }
                    return Change::Partial;
                }
            }
        }
        Change::None
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        let w = canvas.bounds().w;
        header(canvas, "Tidefork", "Done");
        let state = self.game.status();
        let moves = self.game.moves_left();
        let move_label = if moves == 1 { "move" } else { "moves" };
        let goal = self.game.puzzle().target;
        canvas.card(screen_rect(w, 40, 164, 1184, 111));
        canvas.text(
            screen_rect(w, 69, 181, 316, 76),
            &format!("Puzzle {} of {}", self.game.level() + 1, PUZZLES.len()),
            CAPTION,
        );
        canvas.text(
            screen_rect(w, 377, 175, 510, 87),
            &format!("Make {goal}"),
            TextStyle::new(53).bold().center(),
        );
        canvas.text(
            screen_rect(w, 910, 181, 285, 76),
            &format!("{moves} {move_label} left"),
            BODY.bold().right(),
        );

        let guidance = self.notice.unwrap_or(match state {
            Status::Playing => "Tap an arrow to slide its row or column.",
            Status::Complete => "Puzzle complete.",
            Status::OutOfMoves => "Out of moves. Undo a move or reset.",
        });
        canvas.text(screen_rect(w, 40, 284, 1184, 52), guidance, BODY.center());

        canvas.panel(screen_rect(w, 192, 478, 880, 880), 26, 3, PAPER);
        for (i, &value) in self.game.board().iter().enumerate() {
            let x = 192 + (i % SIZE) as i32 * 220;
            let y = 478 + (i / SIZE) as i32 * 220;
            let tile = screen_rect(w, x + 12, y + 12, 196, 196);
            let ink = if value != 0 && value >= goal {
                PAPER
            } else {
                BLACK
            };
            let inside = if ink == PAPER {
                BLACK
            } else if value == 0 {
                247
            } else {
                PAPER
            };
            canvas.panel(tile, 20, if value == 0 { 1 } else { 4 }, inside);
            if value > 0 {
                canvas.text(
                    screen_rect(w, x + 23, y + 58, 174, 112),
                    &value.to_string(),
                    TextStyle::new(76).bold().center().ink(ink),
                );
            }
        }
        for direction in Direction::ALL {
            for lane in 0..SIZE {
                let bounds = arrow_button(w, direction, lane);
                let active = state == Status::Playing;
                let pressed = active && canvas.control_pressed(bounds, false);
                if active {
                    canvas.panel(bounds, 16, 4, if pressed { BLACK } else { PAPER });
                }
                let color = if !active {
                    85
                } else if pressed {
                    PAPER
                } else {
                    BLACK
                };
                canvas.icon(
                    centered_icon(bounds, canvas.px(48)),
                    arrow(direction),
                    color,
                );
            }
        }
        if self.game.can_undo() {
            canvas.button(undo_button(w), "Undo", false, false);
            let action = if state == Status::Complete {
                if self.game.level() + 1 == PUZZLES.len() {
                    "Play again"
                } else {
                    "Next puzzle"
                }
            } else {
                "Reset puzzle"
            };
            canvas.button(action_button(w), action, state == Status::Complete, false);
        }
        if self.confirming_reset {
            reset_sheet(canvas.bounds()).draw(canvas, None);
        }
    }

    fn input(&mut self, input: Input) -> Change {
        match input {
            Input::Tap(x, y) => self.tap(x, y),
            Input::PageBack if self.confirming_reset => {
                self.confirming_reset = false;
                Change::Full
            }
            Input::PageBack => Change::Exit,
            _ => Change::None,
        }
    }

    fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.path {
            primitive_app::save(path, &serde_json::to_vec(self.game.progress())?)?;
        }
        Ok(())
    }
}

fn arrow(direction: Direction) -> Icon {
    let stroke: &[&str] = match direction {
        Direction::Left => &["M21 12H3 M10 5L3 12L10 19"],
        Direction::Right => &["M3 12H21 M14 5L21 12L14 19"],
        Direction::Up => &["M12 21V3 M5 10L12 3L19 10"],
        Direction::Down => &["M12 3V21 M5 14L12 21L19 14"],
    };
    Icon { stroke, fill: &[] }
}

fn centered_icon(bounds: Rect, size: i32) -> Rect {
    Rect::new(
        bounds.x + (bounds.w - size) / 2,
        bounds.y + (bounds.h - size) / 2,
        size,
        size,
    )
}

fn arrow_button(width: i32, direction: Direction, lane: usize) -> Rect {
    let offset = lane as i32 * 220;
    let (x, y, w, h) = match direction {
        Direction::Left => (40, 533 + offset, 112, 110),
        Direction::Right => (1112, 533 + offset, 112, 110),
        Direction::Up => (233 + offset, 348, 138, 110),
        Direction::Down => (233 + offset, 1378, 138, 110),
    };
    screen_rect(width, x, y, w, h)
}

fn undo_button(width: i32) -> Rect {
    screen_rect(width, 40, 1531, 567, 112)
}

fn action_button(width: i32) -> Rect {
    screen_rect(width, 655, 1531, 569, 112)
}

fn reset_sheet(bounds: Rect) -> Sheet<'static> {
    Sheet::centered(
        bounds,
        "Reset this puzzle?",
        "Your moves for this puzzle will be cleared.",
        "Reset",
    )
}
