use std::collections::{HashMap, HashSet};
use std::ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    pub fn forward(self) -> Self {
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

    pub fn opposite(self) -> Self {
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

    pub fn all() -> Vec<Self> {
        use SideOfStar::*;
        vec![A, B, C, D, E, F]
    }
}

impl Default for SideOfStar {
    fn default() -> Self {
        Self::A
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn is_full(self) -> bool {
        !self.is_empty()
    }
}

impl Default for Spot {
    fn default() -> Self {
        Self::Empty
    }
}

impl PartialEq<Player> for Spot {
    fn eq(&self, other: &Player) -> bool {
        match self {
            Self::Player(player) => player == other,
            _ => false,
        }
    }
}

impl PartialEq<Spot> for Player {
    fn eq(&self, spot: &Spot) -> bool {
        spot == self
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

impl ops::Mul<i32> for HexCoord {
    type Output = Self;
    fn mul(self, other: i32) -> Self {
        Self::new(self.horz * other, self.slant * other)
    }
}

impl HexCoord {
    pub const fn new(horz: i32, slant: i32) -> Self {
        HexCoord { horz, slant }
    }

    const NEIGHBOR_OFFSETS: [HexCoord; 6] = [
        Self::new(1, 0),
        Self::new(1, -1),
        Self::new(0, -1),
        Self::new(-1, 0),
        Self::new(-1, 1),
        Self::new(0, 1),
    ];

    fn neighbors(self) -> Vec<Self> {
        Self::NEIGHBOR_OFFSETS
            .iter()
            .copied()
            .map(|neighbor_offset| self + neighbor_offset)
            .collect()
    }

    fn jump_neighbors(self) -> Vec<Self> {
        Self::NEIGHBOR_OFFSETS
            .iter()
            .copied()
            .map(|coord| coord * 2)
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

    pub fn setup_players(&mut self) {
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

        for player in Player::all() {
            let player_exists = self.players.contains(&player);
            let player_tip = player_tip(player);
            for player_tip_coord in player_tip {
                if player_exists {
                    self.put_player(player_tip_coord, player);
                } else {
                    self.remove_player(player_tip_coord);
                }
            }
        }

        self.turn = self.players.iter().min().unwrap().clone();
    }

    pub fn put_player(&mut self, coord: HexCoord, player: Player) {
        if !self.is_valid(coord) {
            return;
        }
        self.board.insert(coord, Spot::Player(player));
    }

    pub fn remove_player(&mut self, coord: HexCoord) {
        if !self.is_valid(coord) {
            return;
        }
        self.board.insert(coord, Spot::Empty);
    }

    pub fn get(&self, coord: &HexCoord) -> Option<&Spot> {
        self.board.get(coord)
    }

    fn is_valid(&self, coord: HexCoord) -> bool {
        self.board.contains_key(&coord)
    }

    fn swap(&mut self, coord1: HexCoord, coord2: HexCoord) {
        if !self.is_valid(coord1) || !self.is_valid(coord2) {
            return;
        }
        let spot1 = self.board.get(&coord1).unwrap().clone();
        let spot2 = self.board.get(&coord2).unwrap().clone();

        self.board.insert(coord1, spot2);
        self.board.insert(coord2, spot1);
    }

    pub fn make_move(&mut self, start_coord: HexCoord, end_coord: HexCoord) -> bool {
        let is_move_valid = self.validate_move(start_coord, end_coord);
        if is_move_valid {
            self.swap(start_coord, end_coord);
        }
        is_move_valid
    }

    fn validate_move(&self, start_coord: HexCoord, end_coord: HexCoord) -> bool {
        let start_spot = self.get(&start_coord);
        let end_spot = self.get(&end_coord);
        if (start_spot.is_none() || end_spot.is_none())
            || (start_spot.unwrap().is_empty() || end_spot.unwrap().is_full())
        // the unwraps never get called on a None due to short-circuit evaluation
        {
            return false;
        }

        if start_coord.neighbors().contains(&end_coord) {
            // is a shift of one place
            true
        } else {
            let mut jump_centers = vec![start_coord];
            let mut traversed_jump_centers = Vec::new();
            while !jump_centers.is_empty() {
                let mut new_jump_centers = Vec::new();
                for &jump_center in &jump_centers {
                    for (neighbor, jump_neighbor) in jump_center
                        .neighbors()
                        .into_iter()
                        .zip(jump_center.jump_neighbors())
                    {
                        if !traversed_jump_centers.contains(&jump_neighbor) {
                            if let (Some(Spot::Player(_)), Some(Spot::Empty)) =
                                (self.get(&neighbor), self.get(&jump_neighbor))
                            {
                                if jump_neighbor == end_coord {
                                    return true;
                                }
                                new_jump_centers.push(jump_neighbor);
                            }
                        }
                    }
                }
                traversed_jump_centers.append(&mut jump_centers);
                jump_centers = new_jump_centers;
            }
            false
        }
    }

    pub fn start_next_turn(&mut self) {
        let mut next_turn = self.turn.forward();
        while !self.players.contains(&next_turn) {
            next_turn = next_turn.forward();
        }
        self.turn = next_turn;
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
    use maplit::hashset;
    use SideOfStar::*;
    let players_count = match players_count {
        2 | 3 | 4 | 6 => players_count,
        _ => 2,
    };

    match players_count {
        2 => hashset![A, D],
        3 => hashset![A, C, E],
        4 => hashset![A, B, D, E],
        6 => hashset![A, B, C, D, E, F],
        _ => unreachable!(),
    }
}
