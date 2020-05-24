use coffee::graphics::{
    Color, Frame, HorizontalAlignment, Mesh, Point, Shape, Transformation, Window, WindowSettings,
};
use coffee::ui::{button, Align, Button, Checkbox, Column, Element, Renderer, Text, UserInterface};

use coffee::input::{mouse, ButtonState, Event, Input};
use coffee::load::Task;
use coffee::{Game, Result, Timer};

use crate::board::{Board, HexCoord, Player, SideOfStar, Spot};

const SIN_30_DEG: f32 = 0.5;
const COS_30_DEG: f32 = 0.866025403784439;
const SIDE: f32 = 20.0;

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
        let point = Point::from([point.x - 350.0, point.y - 350.0]);

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

#[derive(Debug, Clone)]
struct LiftedPiece {
    piece_coord: HexCoord,
    current_position: Point,
    dropped_position: Option<Point>,
}

impl LiftedPiece {
    fn is_dropped(&self) -> bool {
        self.dropped_position.is_some()
    }
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
}

#[derive(Debug, Clone, Default)]
struct DragNDrop {
    has_drag_started: bool,
    drag_start_pos: Option<Point>,
    current_drag_pos: Option<Point>,
    is_dropped: bool,
}

impl Input for DragNDrop {
    fn new() -> Self {
        Self::default()
    }

    fn update(&mut self, event: Event) {
        // we only care for mouse events  
        if let Event::Mouse(event) = event {
            if let mouse::Event::CursorMoved { x, y } = event {
                match (
                    self.has_drag_started,
                    self.drag_start_pos.is_some(),
                    self.current_drag_pos.is_some(),
                ) {
                    // drag not started, do nothing
                    (false, _, _) => {}
                    // drag started but postition not recorded
                    (true, false, false) => {
                        self.drag_start_pos = Some([x, y].into());
                    }
                    // drag started and position recorded
                    (true, true, _) => {
                        let point = [x, y].into();
                        if nalgebra::distance(&self.drag_start_pos.unwrap(), &point) > 5.0 {
                            self.current_drag_pos = Some(point);
                        }
                    }
                    // all other combinations are invalid
                    _ => unreachable!(),
                }
            } else if let mouse::Event::Input { button, state } = event {
                if let mouse::Button::Left = button {
                    match state {
                        ButtonState::Pressed => {
                            self.has_drag_started = true;
                        }
                        ButtonState::Released => {
                            if self.has_drag_started && self.current_drag_pos.is_some() {
                                self.is_dropped = true;
                            }
                        }
                    }
                }
            }
        }
    }

    fn clear(&mut self) {
        if self.is_dropped {
            *self = Self::default();
        }
    }
}

impl DragNDrop {
}

#[derive(Clone, Debug)]
struct BoardGame {
    inner_board: Board,
    center: [f32; 2],
    phase: Phase,
    lifted_piece: Option<LiftedPiece>,
    start_button_state: button::State,
}

impl BoardGame {
    fn new() -> Self {
        Self {
            inner_board: Board::new(3),
            center: [450.0, 350.0],
            phase: Phase::Start,
            lifted_piece: None,
            start_button_state: button::State::default(),
        }
    }

    fn circle_mesh(&self) -> Mesh {
        let circle_centers = self
            .inner_board
            .board
            .iter()
            .map(|(&coord, &spot)| (coord.hexagon_center(SIDE), spot.color()));

        let mut mesh = Mesh::new();

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
        let circles = self.circle_mesh();

        {
            let transformation = Transformation::translate(self.center.into());
            let mut grid_target = target.transform(transformation);
            circles.draw(&mut grid_target);
        }
    }

    fn interact(&mut self, input: &mut Self::Input, window: &mut Window) {
        let width_scale = 0.5;
        let height_scale = 0.5;
        self.center = [window.width() * width_scale, window.height() * height_scale];
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Next,
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
            .spacing(30)
            .width(window.width() as u32)
            .push(heading);
        let next_button = Button::new(&mut self.start_button_state, "Next")
            .width(350)
            .on_press(Message::Next);

        column = match self.phase {
            Phase::Start => {
                let description_text = "\
                This is a game of Chinese Checkers. \
                This game was originally called Stern-halma or sth like that in German. \
                ";
                let description = Text::new(description_text).width(500);

                column.push(description).push(next_button)
            }
            Phase::GameSetup => {
                let mut checkboxes = Column::new().spacing(5).width(400);
                for side_of_star in SideOfStar::all() {
                    let label = &format!("Player  {:?}", side_of_star);
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
                column.push(sub_heading).push(checkboxes).push(next_button)
            }
            Phase::GamePlay => {
                let turn = Text::new(&format!("Turn: {:?}", self.inner_board.turn)).size(25);
                column.push(turn)
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
