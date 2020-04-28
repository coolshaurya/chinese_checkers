use std::collections::{HashMap, HashSet};
use std::ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideOfStar {
    A,
    B,
    C,
    D,
    E,
    F,
}

pub type Player = SideOfStar;

impl SideOfStar {
    fn forward(self) -> Self {
        use SideOfStar::*;
        match self {
            A => B,
            B => C,
            C => D,
            D => E,
            E => F,
            F => A,
        }
    }

    fn opposite(self) -> Self {
        use SideOfStar::*;
        match self {
            A => D,
            B => E,
            C => F,
            D => A,
            E => B,
            F => C,
        }
    }
}

impl Default for SideOfStar {
    fn default() -> Self {
        Self::A
    }
}

impl Iterator for SideOfStar {
    type Item = Self;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.forward())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Spot {
    Empty,
    Player(Player),
}

impl Spot {
    pub fn is_empty(self) -> bool {
        match self {
            Self::Empty => true,
            Self::Player(_) => false,
        }
    }

    pub fn is_player(self, player: Player) -> bool {
        match self {
            Self::Player(current_player) => player == current_player,
            Self::Empty => false,
        }
    }
}

impl Default for Spot {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct HexCoord {
    /// the horizontal axis
    pub horz: i32,
    /// the slant axis
    pub slant: i32,
}

impl From<(i32, i32)> for HexCoord {
    fn from(other: (i32, i32)) -> Self {
        Self::new(other.0, other.1)
    }
}

impl ops::Add for HexCoord {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.horz + other.horz, self.slant + other.slant)
    }
}

impl ops::Mul<i32> for HexCoord {
    type Output = Self;
    fn mul(self, other: i32) -> Self {
        Self::new(self.horz * other, self.slant * other)
    }
}

impl ops::Neg for HexCoord {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.horz, -self.slant)
    }
}

impl ops::Sub for HexCoord {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

impl HexCoord {
    pub const fn new(horz: i32, slant: i32) -> Self {
        HexCoord { horz, slant }
    }

    fn neighbors(self) -> Vec<Self> {
        vec![
            Self::new(0, -1),
            Self::new(0, 1),
            Self::new(1, 0),
            Self::new(-1, 0),
            Self::new(-1, 1),
            Self::new(1, -1),
        ]
        .into_iter()
        .map(|neighbor_offset| self + neighbor_offset)
        .collect()
    }

    fn triangle_tip_up(self, size: i32) -> Vec<Self> {
        (0_i32..size)
            .map(|offset| (offset, self.slant + offset))
            .flat_map(|(offset, slant)| {
                (0_i32..size)
                    .take((offset + 1) as usize)
                    .map(move |offset_nested| self.horz - offset_nested)
                    .map(move |horz| Self::new(horz, slant))
            })
            .collect()
    }

    fn triangle_tip_down(self, size: i32) -> Vec<Self> {
        (0_i32..size)
            .map(|offset| (offset, self.slant - offset))
            .flat_map(|(offset, slant)| {
                (0_i32..size)
                    .take((offset + 1) as usize)
                    .map(move |offset_nested| self.horz + offset_nested)
                    .map(move |horz| Self::new(horz, slant))
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
enum Move {
    Shift(HexCoord),
    Jump(Vec<HexCoord>),
}

#[derive(Debug, Clone, Default)]
struct GameOutcome(Vec<Player>);

#[derive(Debug, Clone)]
pub struct Board {
    pub board: HashMap<HexCoord, Spot>,
    pub players: HashSet<Player>,
    pub turn: Player,
}

impl Board {
    pub fn new(players_count: usize) -> Self {
        let mut new_board = Self {
            players: gen_players(players_count),
            board: gen_board(),
            turn: Player::default(),
        };

        new_board.setup_players();

        new_board
    }

    fn setup_players(&mut self) {
        let player_tip = |side_of_star: SideOfStar| -> Vec<HexCoord> {
            use SideOfStar::*;
            match side_of_star {
                A => HexCoord::new(4, -8).triangle_tip_up(4),
                B => HexCoord::new(5, -1).triangle_tip_down(4),
                C => HexCoord::new(4, 1).triangle_tip_up(4),
                D => HexCoord::new(-4, 8).triangle_tip_down(4),
                E => HexCoord::new(-5, 1).triangle_tip_up(4),
                F => HexCoord::new(-4, -1).triangle_tip_down(4),
            }
        };

        for &player in self.players.clone().iter() {
            let player_tip = player_tip(player);
            for player_tip_coord in player_tip {
                self.put_player(player_tip_coord, player);
            }
        }
    }

    pub fn put_player(&mut self, coord: HexCoord, player: Player) {
        if !self.is_valid(coord) {
            return;
        }
        self.board.insert(coord, Spot::Player(player));
    }

    pub fn get(&self, coord: &HexCoord) -> Option<&Spot> {
        self.board.get(coord)
    }

    pub fn remove_player(&mut self, coord: HexCoord) {
        if !self.is_valid(coord) {
            return;
        }
        self.board.insert(coord, Spot::Empty);
    }

    fn is_valid(&self, coord: HexCoord) -> bool {
        self.board.contains_key(&coord)
    }

    pub fn swap(&mut self, coord1: HexCoord, coord2: HexCoord) {
        if !self.is_valid(coord1) || !self.is_valid(coord2) {
            return;
        }
        let spot1 = self.board.get(&coord1).unwrap().clone();
        let spot2 = self.board.get(&coord2).unwrap().clone();

        self.board.insert(coord1, spot2);
        self.board.insert(coord2, spot1);
    }
}

fn gen_board() -> HashMap<HexCoord, Spot> {
    let big_triangle = HexCoord::new(4, -8).triangle_tip_up(13).into_iter();

    let triangle_1 = HexCoord::new(-4, -1).triangle_tip_down(4).into_iter();
    let triangle_2 = HexCoord::new(5, -1).triangle_tip_down(4).into_iter();
    let triangle_3 = HexCoord::new(-4, 8).triangle_tip_down(4).into_iter();

    big_triangle
        .chain(triangle_1)
        .chain(triangle_2)
        .chain(triangle_3)
        .map(|coord| (coord, Spot::default()))
        .collect()
}

fn gen_players(players_count: usize) -> HashSet<Player> {
    use SideOfStar::*;
    let players_count = match players_count {
        2 | 3 | 4 | 6 => players_count,
        _ => 2,
    };

    match players_count {
        2 => vec![A, D].into_iter().collect(),
        3 => vec![A, C, E].into_iter().collect(),
        4 => vec![A, B, D, E].into_iter().collect(),
        6 => vec![A, B, C, D, E, F].into_iter().collect(),
        _ => unreachable!(),
    }
}
