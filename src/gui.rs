use coffee::graphics::{
    Color, CursorIcon, Frame, HorizontalAlignment, Mesh, Point, Shape, Transformation, Window,
    WindowSettings,
};
use coffee::ui::{
    button, Align, Button, Checkbox, Column, Element, Justify, Renderer, Row, Text, UserInterface,
};

use coffee::load::Task;
use coffee::{Game, Result, Timer};

use crate::board::{Board, HexCoord, Player, SideOfStar, Spot};

mod dragndrop;
use dragndrop::DragNDrop;

const SIN_30_DEG: f32 = 0.5;
const COS_30_DEG: f32 = 0.866025403784439;
const SIDE: f32 = 22.0;

impl SideOfStar {
    fn color(self) -> Color {
        match self {
            Player::A => Color::from_rgb_u32(0xEE1133),
            Player::B => Color::from_rgb_u32(0xFFE122),
            Player::C => Color::from_rgb_u32(0xEE22CC),
            Player::D => Color::from_rgb_u32(0x22EE55),
            Player::E => Color::from_rgb_u32(0x2255FF),
            Player::F => Color::from_rgb_u32(0xAA22FF),
        }
    }
}

impl Spot {
    fn color(self) -> Color {
        match self {
            Self::Empty => Color::from_rgb_u32(0xEEEEEE),
            Self::Player(player) => player.color(),
        }
    }
}

impl HexCoord {
    fn hexagon_center(self, side: f32) -> Point {
        let horz: f32 = self.horz as f32;
        let slant: f32 = self.slant as f32;
        let x: f32 = side * COS_30_DEG * (2.0 * horz + slant);
        let y: f32 = slant * (side * SIN_30_DEG + side);
        [x, y].into()
    }

    // NOTE: all the code in this fn has been directly copied from
    // https://www.redblobgames.com/grids/hexagons/. This code uses
    // the pixel to hex conversion and then rounds it. In the
    // rounding code the redundant `rz` has been removed where
    // nescessary.
    fn from_point(point: Point, side: f32) -> Self {
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
    hexagon_side * COS_30_DEG * 0.85
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Start,
    GameSetup,
    GamePlay,
}

impl Phase {
    fn next(self) -> Self {
        use Phase::*;
        match self {
            Start => GameSetup,
            GameSetup => GamePlay,
            GamePlay => unreachable!(),
        }
    }
    fn previous(self) -> Self {
        use Phase::*;
        match self {
            Start => unreachable!(),
            GameSetup => Start,
            GamePlay => GameSetup,
        }
    }
}
#[derive(Debug, Clone)]
pub struct LiftedPiece {
    pub piece_coord: HexCoord,
    pub current_pos: Point,
    pub dropped_coord: Option<HexCoord>,
}

impl LiftedPiece {
    pub fn update_pos(&mut self, new_pos: Point) {
        self.current_pos = new_pos;
    }
    pub fn new(coord: HexCoord, current_pos: Point) -> Self {
        Self {
            piece_coord: coord,
            current_pos,
            dropped_coord: None,
        }
    }
    pub fn drop_piece<F>(&mut self, make_coord: F)
    where
        F: FnOnce(Point) -> HexCoord,
    {
        self.dropped_coord = Some(make_coord(self.current_pos));
    }
}

#[derive(Clone, Debug)]
struct BoardGame {
    inner_board: Board,
    grid_center: [f32; 2],
    phase: Phase,
    lifted_piece: Option<LiftedPiece>,
    next_button_state: button::State,
    previous_button_state: button::State,
}

impl BoardGame {
    fn new() -> Self {
        Self {
            inner_board: Board::new(3),
            grid_center: [450.0, 350.0],
            phase: Phase::Start,
            lifted_piece: None,
            next_button_state: button::State::default(),
            previous_button_state: button::State::default(),
        }
    }

    fn circle_mesh(&self) -> Mesh {
        let circle_centers = self
            .inner_board
            .board
            .iter()
            .map(|(&coord, &spot)| (coord.hexagon_center(SIDE), spot.color()));

        let mut mesh = Mesh::new_with_tolerance(0.05);

        for (circle_center, color) in circle_centers {
            mesh.fill(
                Shape::Circle {
                    center: circle_center,
                    radius: ideal_radius(SIDE),
                },
                color,
            )
        }

        mesh
    }
}

impl Game for BoardGame {
    type Input = DragNDrop;
    type LoadingScreen = ();

    fn load(_window: &Window) -> Task<Self> {
        Task::succeed(|| Self::new())
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::BLACK);

        if self.phase != Phase::GamePlay {
            return;
        }

        let mut target = frame.as_target();

        {
            let transformation = Transformation::translate(self.grid_center.into());
            let mut grid_target = target.transform(transformation);

            let circles = self.circle_mesh();
            circles.draw(&mut grid_target);

            if let Some(lifted_piece) = &self.lifted_piece {
                let circle = |center| Shape::Circle {
                    center,
                    radius: ideal_radius(SIDE),
                };
                let change_alpha = |color: Color| Color::new(color.r, color.g, color.b, 0.9);

                let mut dragndrop_mesh = Mesh::new();
                let spot = self
                    .inner_board
                    .get(&lifted_piece.piece_coord)
                    .unwrap()
                    .clone();

                let lifted_indicator = circle(lifted_piece.piece_coord.hexagon_center(SIDE));
                let lifted_indicator_color = match spot {
                    Spot::Player(player) => {
                        if player < Player::D {
                            Color::BLUE
                        } else {
                            Color::RED
                        }
                    }
                    _ => unreachable!()
                };
                dragndrop_mesh.fill(lifted_indicator.clone(), Spot::Empty.color());
                dragndrop_mesh.stroke(lifted_indicator, lifted_indicator_color, 4.0);

                let floating_circle = circle(lifted_piece.current_pos);
                let floating_circle_color = change_alpha(spot.color());
                dragndrop_mesh.fill(floating_circle, floating_circle_color);

                dragndrop_mesh.draw(&mut grid_target);
            }
        }
    }

    fn interact(&mut self, input: &mut Self::Input, window: &mut Window) {
        let width_offset_percentage = 0.5;
        let height_offset_percentage = 0.5;
        self.grid_center = [
            window.width() * width_offset_percentage,
            window.height() * height_offset_percentage,
        ];

        let grid_center = self.grid_center;
        let make_point_relative =
            |point: Point| -> Point { point - Point::from(grid_center).coords };
        let point_to_coord = |point: Point| HexCoord::from_point(point, SIDE);

        if let Some((current_drag_pos, start_drag_pos)) = input.drag_status() {
            let current_drag_pos = make_point_relative(current_drag_pos);
            if let Some(lifted_piece) = self.lifted_piece.as_mut() {
                lifted_piece.update_pos(current_drag_pos);
                if input.is_dropped() {
                    lifted_piece.drop_piece(point_to_coord);
                }
            } else {
                let start_coord = point_to_coord(make_point_relative(start_drag_pos));
                let spot = self.inner_board.get(&start_coord);
                if let Some(spot) = spot {
                    if spot.is_full() {
                        self.lifted_piece = Some(LiftedPiece::new(start_coord, current_drag_pos));
                    }
                }
            }
        }
    }

    fn update(&mut self, _window: &Window) {
        if self.phase != Phase::GamePlay {
            return;
        }
        if let Some(lifted_piece) = &self.lifted_piece {
            if let Some(dropped_coord) = lifted_piece.dropped_coord {
                if self
                    .inner_board
                    .make_move(lifted_piece.piece_coord, dropped_coord)
                {
                    self.inner_board.start_next_turn();
                };
                self.lifted_piece = None;
            }
        }
    }

    fn cursor_icon(&self) -> CursorIcon {
        if self.lifted_piece.is_some() {
            CursorIcon::Move
        } else {
            CursorIcon::Default
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Next,
    Previous,
    PlayerToggle(SideOfStar, bool),
}

impl UserInterface for BoardGame {
    type Renderer = Renderer;
    type Message = Message;

    fn layout(&mut self, window: &Window) -> Element<Self::Message> {
        let heading = Text::new("Chinese Checkers")
            .horizontal_alignment(HorizontalAlignment::Center)
            .size(80);

        let mut column = Column::new()
            .align_items(Align::Center)
            .spacing(20)
            .width(window.width() as u32);
        let next_button = Button::new(&mut self.next_button_state, "Next")
            .width(350)
            .on_press(Message::Next);
        let previous_button = Button::new(&mut self.previous_button_state, "Previous")
            .width(350)
            .on_press(Message::Previous);

        column = match self.phase {
            Phase::Start => {
                let description_text = "\
                This is a game of Chinese Checkers. \
                Chinese Checkers originated in Germany where it was called Sternhalma. \
                Chinese Checkers is played on a star-shaped board. \
                ";
                let description = Text::new(description_text).width(500);

                column.push(heading).push(description).push(next_button)
            }
            Phase::GameSetup => {
                let mut checkboxes = Column::new().spacing(5).width(400);
                for side_of_star in SideOfStar::all() {
                    let label = &format!("Side{:?}", side_of_star);
                    let checkbox = Checkbox::new(
                        self.inner_board.players.contains(&side_of_star),
                        label,
                        move |checked| Message::PlayerToggle(side_of_star, checked),
                    );
                    checkboxes = checkboxes.push(checkbox);
                }

                let sub_heading = Text::new("Please select the players you want")
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .size(30);
                column
                    .push(heading)
                    .push(sub_heading)
                    .push(checkboxes)
                    .push(next_button)
                    .push(previous_button)
            }
            Phase::GamePlay => {
                let mut spacer_column = Column::new()
                    .justify_content(Justify::SpaceBetween)
                    .align_items(Align::Center)
                    .spacing((window.height() * 0.8) as u16);
                let turn = Row::new().justify_content(Justify::SpaceAround).push(
                    Text::new(&format!("Turn:{:?}", self.inner_board.turn))
                        .size(25)
                        .horizontal_alignment(HorizontalAlignment::Center),
                );
                spacer_column = spacer_column.push(turn).push(previous_button);
                let heading = heading.size(40);
                column.spacing(10).push(heading).push(spacer_column)
            }
        };

        column.into()
    }

    fn react(&mut self, message: Self::Message, _window: &mut Window) {
        match message {
            Message::Next => {
                if self.phase == Phase::GameSetup {
                    self.inner_board.setup_players();
                };
                self.phase = self.phase.next();
            }
            Message::Previous => {
                self.phase = self.phase.previous();
            }
            Message::PlayerToggle(side, checked) => {
                if checked {
                    self.inner_board.players.insert(side);
                } else {
                    self.inner_board.players.remove(&side);
                }
            }
        }
    }
}

pub fn start_game() -> Result<()> {
    <BoardGame as UserInterface>::run(WindowSettings {
        title: String::from("Chinese Checkers"),
        size: (900, 700),
        resizable: true,
        fullscreen: false,
        maximized: true,
    })
}
