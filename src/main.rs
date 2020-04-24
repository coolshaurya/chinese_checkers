mod board;
mod gui;

use ggez::GameResult;

fn main() -> GameResult {
    gui::start_game()?;

    Ok(())
}
