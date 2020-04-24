use ggez::conf::NumSamples;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, mint, Canvas, DrawMode, DrawParam, MeshBuilder};
use ggez::{Context, ContextBuilder, GameResult};

use crate::board::{Board, HexCoord};

impl HexCoord {
    fn to_point2(self, side: f32) -> mint::Point2<f32> {
        const SIN_30_DEG: f32 = 0.5;
        const COS_30_DEG: f32 = 0.866025403784439;
        let horz: f32 = self.horz as f32;
        let slant: f32 = self.slant as f32;
        let x: f32 = side * COS_30_DEG * (2.0 * horz + slant);
        let y: f32 = slant * (side * SIN_30_DEG + side);
        println!("({}, {})", x, y);
        [x, y].into()
    }
}

#[derive(Clone, Debug)]
struct BoardState {
    board: Board,
}

impl BoardState {
    fn new() -> Self {
        Self {
            board: Board::new(2),
        }
    }
}

impl EventHandler for BoardState {
    fn update(&mut self, context: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, context: &mut Context) -> GameResult {
        graphics::clear(context, graphics::BLACK);

        let circle_centers = self.board.board.keys().map(|&coord| coord.to_point2(22.0));
        let mut mesh = MeshBuilder::new();

        for circle_center in circle_centers {
            mesh.circle(DrawMode::fill(), circle_center, 18.0, 0.1, graphics::WHITE);
        }

        let mesh = mesh.build(context)?;

        graphics::draw(context, &mesh, DrawParam::default().dest([300.0, 300.0]))?;

        graphics::present(context)?;

        Ok(())
    }
}

pub fn start_game() -> GameResult {
    let (mut context, mut events_loop) =
        ContextBuilder::new("chinese_checkers", "coolshaurya").build()?;
    event::run(&mut context, &mut events_loop, &mut BoardState::new())?;

    Ok(())
}
