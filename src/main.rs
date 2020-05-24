mod board;
mod gui;

use coffee::Result;
use gui::start_game;

fn main() -> Result<()> {
    start_game()
}
