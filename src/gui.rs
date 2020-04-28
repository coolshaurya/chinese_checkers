use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, mint::Point2, Color, DrawMode, DrawParam, Mesh, MeshBuilder};
use ggez::input::mouse::{self, MouseButton, MouseCursor};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};

use std::time::{Duration, Instant};

use crate::board::{Board, HexCoord, Player, SideOfStar, Spot};

const SIN_30_DEG: f32 = 0.5;
const COS_30_DEG: f32 = 0.866025403784439;
const SIDE: f32 = 25.0;

impl Spot {
    fn color(self) -> Color {
        match self {
            Self::Empty => Color::from_rgb_u32(0xEEEEEE),
            Self::Player(player) => match player {
                Player::A => Color::from_rgb_u32(0xEE1133),
                Player::B => Color::from_rgb_u32(0xFFE122),
                Player::C => Color::from_rgb_u32(0xEE22CC),
                Player::D => Color::from_rgb_u32(0x22EE55),
                Player::E => Color::from_rgb_u32(0x2255FF),
                Player::F => Color::from_rgb_u32(0xAA22FF),
            },
        }
    }
}

impl HexCoord {
    fn hexagon_center(self, side: f32) -> Point2<f32> {
        let horz: f32 = self.horz as f32;
        let slant: f32 = self.slant as f32;
        let x: f32 = side * COS_30_DEG * (2.0 * horz + slant);
        let y: f32 = slant * (side * SIN_30_DEG + side);
        [x, y].into()
    }

    // NOTE: all the code in this fn has been directly copied from
    // https://www.redblobgames.com/grids/hexagons/ it uses the
    // pixel to hex conversion and then rounds it. In the rounding
    // code the redundant `rz` has been removed where nescessary
    fn from_point(point: Point2<f32>, side: f32) -> Self {
        let point = Point2::from([point.x - 350.0, point.y - 350.0]);

        let horz = (3.0_f32.sqrt() / 3.0 * point.x - 1.0 / 3.0 * point.y) / side;
        let slant = (2.0 / 3.0 * point.y) / side;

        let x = horz;
        let y = slant;
        let z = -horz - slant;

        let mut rx = x.round();
        let mut ry = y.round();
        let rz = z.round();

        let x_diff = (rx - x).abs();
        let y_diff = (ry - y).abs();
        let z_diff = (rz - z).abs();

        if x_diff > y_diff && x_diff > z_diff {
            rx = -ry - rz
        } else if y_diff > z_diff {
            ry = -rx - rz
        }

        Self::new(rx as i32, ry as i32)
    }
}

fn ideal_radius(hexagon_side: f32) -> f32 {
    hexagon_side * COS_30_DEG * 0.97
}

#[derive(Clone, Debug)]
struct BoardGui {
    board: Board,
}

impl BoardGui {
    fn new() -> Self {
        Self {
            board: Board::new(2),
        }
    }

    fn circle_mesh(&self, context: &mut Context) -> GameResult<Mesh> {
        let circle_centers = self
            .board
            .board
            .iter()
            .map(|(&coord, &spot)| (coord.hexagon_center(SIDE), spot.color()));

        let mut mesh = MeshBuilder::new();

        for (circle_center, color) in circle_centers {
            mesh.circle(
                DrawMode::fill(),
                circle_center,
                ideal_radius(SIDE),
                0.1,
                color,
            );
        }

        mesh.build(context)
    }
}

impl EventHandler for BoardGui {
    fn update(&mut self, _context: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, context: &mut Context) -> GameResult {
        graphics::clear(context, graphics::BLACK);

        let highlighted = HexCoord::from_point(mouse::position(context), SIDE);

        if mouse::button_pressed(context, MouseButton::Left) {
            mouse::set_cursor_type(context, MouseCursor::Grabbing);
        } else {
            mouse::set_cursor_type(context, MouseCursor::Default);
        }

        let circles = self.circle_mesh(context)?;

        graphics::draw(context, &circles, DrawParam::default().dest([350.0, 350.0]))?;

        graphics::present(context)?;
        timer::yield_now();

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _context: &mut Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.last_click = Some(Instant::now());
        println!("mouse pressed!");
    }
    fn mouse_button_up_event(
        &mut self,
        _context: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        println!("mouse released!");
    }
}

pub fn start_game() -> GameResult {
    let (mut context, mut events_loop) = ContextBuilder::new("chinese_checkers", "coolshaurya")
        .window_setup(WindowSetup::default().title("Chinese Checkers"))
        .window_mode(WindowMode::default().dimensions(700.0, 700.0))
        .build()?;
    event::run(&mut context, &mut events_loop, &mut BoardGui::new())?;

    Ok(())
}
