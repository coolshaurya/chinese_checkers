use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerId {
    A,
    B,
    C,
    D,
    E,
    F,
}

impl PlayerId {
    fn forward(self) -> Self {
        use PlayerId::*;
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
        use PlayerId::*;
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

impl Default for PlayerId {
    fn default() -> Self {
        Self::A
    }
}

impl Iterator for PlayerId {
    type Item = Self;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.forward())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Spot {
    Empty,
    Player(PlayerId),
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

impl HexCoord {
    const fn new(horz: i32, slant: i32) -> Self {
        HexCoord { horz, slant }
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

/*
#[derive(Debug, Clone)]
enum GameOutcome {
    Indefinite,
    WinSequence(Vec<PlayerId>),
    Finished(Vec<PlayerId>),
}*/

#[derive(Debug, Clone)]
pub struct Board {
    pub board: HashMap<HexCoord, Spot>,
    pub players: HashSet<PlayerId>,
    pub turn: PlayerId,
}

impl Board {
    pub fn new(players_count: usize) -> Self {
        Self {
            players: gen_players(players_count),
            board: gen_board(),
            turn: PlayerId::default(),
        }
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

fn gen_players(players_count: usize) -> HashSet<PlayerId> {
    let players_count = match players_count {
        2 | 3 | 4 | 6 => players_count,
        _ => 2,
    };

    let mut players = HashSet::with_capacity(6);
    match players_count {
        2 => {
            players = PlayerId::default()
                .enumerate()
                .take(4)
                .filter(|(index, _)| index % 3 == 0)
                .map(|(_, val)| val)
                .collect();
        }
        3 => {
            players = PlayerId::default()
                .enumerate()
                .take(5)
                .filter(|(index, _)| index % 2 == 0)
                .map(|(_, val)| val)
                .collect();
        }
        4 => {
            players = PlayerId::default()
                .take(2)
                .flat_map(|val| vec![val, val.opposite()])
                .collect();
        }
        6 => {
            players = PlayerId::default().take(6).collect();
        }
        _ => unreachable!(),
    }

    players
}
