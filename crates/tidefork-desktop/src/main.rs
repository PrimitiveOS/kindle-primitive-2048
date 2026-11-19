use std::{
    fs,
    path::{Path, PathBuf},
};

use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};
use tidefork_core::Status;
use tidefork_render::View;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let mut view = View::preview();
    let mut capture = |view: &View, name: &str, dark| {
        write_png(
            &output.join(name),
            WIDTH,
            HEIGHT,
            &frame(view, &mut canvas, dark),
        )
    };

    capture(&view, "tidefork.png", false)?;
    assert_eq!(view.input(Input::Tap(90.0, 590.0)), Change::Partial);
    assert_eq!(view.game.moves_left(), 1);
    capture(&view, "tidefork-playing-dark.png", true)?;
    assert_eq!(view.input(Input::Tap(520.0, 402.0)), Change::Partial);
    assert_eq!(view.game.status(), Status::Complete);
    capture(&view, "tidefork-complete.png", false)?;
    assert_eq!(view.input(Input::Tap(920.0, 1586.0)), Change::Full);
    assert_eq!(view.game.level(), 1);
    assert_eq!(view.input(Input::Tap(1110.0, 90.0)), Change::Exit);
    Ok(())
}
