use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::{Display, Write},
};

use crate::vec2d::Vec2D;

use super::{DayOutput, LogicError, PartResult};

const CAVE_WIDTH: i64 = 7;
const ROCK_VERTICAL_SPAWN_OFFSET: i64 = 3;
const ROCK_HORIZONTAL_SPAWN_OFFSET: i64 = 2;

static ROCK_MINUS: Rock = Rock {
    blocks: [
        Vec2D { x: 0, y: 0 },
        Vec2D { x: 1, y: 0 },
        Vec2D { x: 2, y: 0 },
        Vec2D { x: 3, y: 0 },
    ]
    .as_slice(),
    width: 4,
    height: 1,
};

static ROCK_PLUS: Rock = Rock {
    blocks: [
        Vec2D { x: 1, y: 0 },
        Vec2D { x: 1, y: 1 },
        Vec2D { x: 0, y: 1 },
        Vec2D { x: 2, y: 1 },
        Vec2D { x: 1, y: 2 },
    ]
    .as_slice(),
    width: 3,
    height: 3,
};

static ROCK_L: Rock = Rock {
    blocks: [
        Vec2D { x: 0, y: 0 },
        Vec2D { x: 1, y: 0 },
        Vec2D { x: 2, y: 0 },
        Vec2D { x: 2, y: 1 },
        Vec2D { x: 2, y: 2 },
    ]
    .as_slice(),
    width: 3,
    height: 3,
};

static ROCK_PIPE: Rock = Rock {
    blocks: [
        Vec2D { x: 0, y: 0 },
        Vec2D { x: 0, y: 1 },
        Vec2D { x: 0, y: 2 },
        Vec2D { x: 0, y: 3 },
    ]
    .as_slice(),
    width: 1,
    height: 4,
};

static ROCK_CUBE: Rock = Rock {
    blocks: [
        Vec2D { x: 0, y: 0 },
        Vec2D { x: 0, y: 1 },
        Vec2D { x: 1, y: 0 },
        Vec2D { x: 1, y: 1 },
    ]
    .as_slice(),
    width: 2,
    height: 2,
};

static ROCKS: [&Rock; 5] = [&ROCK_MINUS, &ROCK_PLUS, &ROCK_L, &ROCK_PIPE, &ROCK_CUBE];

struct Rock<'a> {
    blocks: &'a [Vec2D<i64>],
    width: i64,
    height: i64,
}

#[derive(Clone, Copy)]
enum Jet {
    Left,
    Right,
}

type FloorShape = [i64; CAVE_WIDTH as usize];

impl From<char> for Jet {
    fn from(value: char) -> Self {
        match value {
            '<' => Self::Left,
            '>' => Self::Right,
            _ => panic!("Unexpected input, expected only '>' or '<'"),
        }
    }
}

/// A block of a collection of rocks applied to a tower
/// It can be seen as a map of one tower state to another
/// It requires both rock_index and jet_index be 0 at the "joints"
struct Block {
    height: i64,
    top_shape: FloorShape,
    jet_offset: i64,
    rock_count: i64,
}

struct RockTower<'a> {
    rock_iter_pos: usize,
    jet_iter_pos: usize,
    rocks_to_rest: i64,
    jets: &'a [Jet],
    floor_map: HashMap<FloorShape, Block>,
    inhibit_superblock: bool, // board: Board,
                              // rock_iter:
                              // std::iter::Cycle<std::iter::Cloned<std::slice::Iter<'static, &'static Rock<'static>>>>,
                              // rock_iter: std::iter::Cycle<std::slice::Iter<'a, &'static Rock<'static>>>,
}

impl<'a> RockTower<'a> {
    fn new(rocks_to_rest: i64, jets: &'a [Jet]) -> Self {
        Self {
            rocks_to_rest,
            jets,
            floor_map: HashMap::new(),
            inhibit_superblock: false,
            rock_iter_pos: 0,
            jet_iter_pos: 0,
            // rock_iter: ROCKS.iter().cloned().cycle(),
            // board: Board::new(ROCKS[0]),
        }
    }

    fn next_rock(&mut self) -> usize {
        (self.rock_iter_pos + 1) % ROCKS.len()
    }

    fn next_jet(&mut self) -> Jet {
        let jet = &self.jets[self.jet_iter_pos];
        self.jet_iter_pos = (self.jet_iter_pos + 1) % self.jets.len();
        *jet
    }

    fn remaining_rocks(&self, board: &Board) -> i64 {
        self.rocks_to_rest - board.resting_rock_count
    }

    fn block_size(&self) -> usize {
        self.jets.len() * ROCKS.len()
    }

    fn calc_tower_height(&mut self) -> i64 {
        let mut board = Board::new(0); // Block here doesn't matter, run_block runs its own iter if needed

        println!(
            "Block size {}x{}={}",
            self.jets.len(),
            ROCKS.len(),
            self.block_size()
        );

        let mut jet_index = 0;

        // Block only
        // while self.remaining_rocks(&board) > self.block_size() as i64 && !self.inhibit_superblock {
        //     // println!("Running block");
        //     self.run_block(&mut board, &mut jet_index);
        //     println!("Stack height now {}", board.stack_height)
        // }

        while self.remaining_rocks(&board) > 0 {
            let jet = *self.jets.get(jet_index).unwrap();

            jet_index = (jet_index + 1) % self.jets.len();

            board.advance(jet);
        }

        board.top + board.stack_height
    }

    // fn create_block(&self, mut start_board: Board) -> Block {

    // }

    fn run_block(&mut self, board: &mut Board, jet_index: &mut usize) {
        let block_size = self.block_size();
        board.insert_new_rock(self.next_rock());

        match self.floor_map.entry(board.field) {
            Entry::Occupied(e) => {
                println!("Using cache");
                let block = e.get();

                board.field = block.top_shape;
                board.stack_height += block.height;
                board.resting_rock_count += self.block_size() as i64;
                board.top = *board.field.iter().max().unwrap();
            }
            Entry::Vacant(e) => {}
        }
        println!("Simulating block");
        let start_height = board.stack_height;
        let block_cap = board.resting_rock_count + block_size as i64;
        loop {
            let rock = self.next_rock();
            let jet = self.next_jet();
            board.advance(jet);
        }
        let end_floor = board.field;
        let end_height = board.stack_height;

        // e.insert(Block {
        //     height: end_height - start_height,
        //     top_shape: end_floor,
        // });
        // }
        // }

        // if let Entry::Vacant(e) =  {

        // } else {

        // }
    }
}

/// State of the not-tetris board
#[derive(Clone)]
struct Board {
    /// Floor shape
    field: FloorShape,

    /// Currently falling rock
    falling_rock: usize,

    /// Position of the bottomleft corner of the falling rock
    falling_rock_position: Vec2D<i64>,

    /// Highest point of the floor, used for determining spawn position
    top: i64,

    /// How many rocks have fallen and rested
    resting_rock_count: i64,

    /// Height "below" the floor, added to by normalizing floor shape
    stack_height: i64,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let top_y = self.falling_rock_position.y + self.rock().height + 1;
        for n in 0..top_y {
            let y = top_y - (n + 1);

            f.write_char('|')?;
            for x in 0..CAVE_WIDTH {
                let charpos = Vec2D { x, y };
                if *self.field.get(charpos.x as usize).unwrap() > charpos.y {
                    f.write_char('#')?;
                } else if self
                    .rock()
                    .blocks
                    .iter()
                    .map(|pos| (*pos + self.falling_rock_position))
                    .any(|pos| pos == charpos)
                {
                    f.write_char('@')?;
                } else {
                    f.write_char('.')?;
                }
            }
            f.write_char('|')?;
            f.write_char('\n')?;
        }

        f.write_str("+-------+")
    }
}

impl Board {
    fn new(start_rock: usize) -> Self {
        let mut a = Self {
            field: [0, 0, 0, 0, 0, 0, 0],
            falling_rock: start_rock,
            falling_rock_position: Vec2D { x: 2, y: 4 },
            top: 0,
            resting_rock_count: 0,
            stack_height: 0,
        };
        a.set_start_position();

        a
    }

    fn rock(&self) -> &'static Rock<'static> {
        ROCKS.get(self.falling_rock).unwrap()
    }

    fn set_start_position(&mut self) {
        self.falling_rock_position.y = self.top + ROCK_VERTICAL_SPAWN_OFFSET;
        self.falling_rock_position.x = ROCK_HORIZONTAL_SPAWN_OFFSET;
    }

    fn next_rock(&self) -> usize {
        (self.falling_rock + 1) % ROCKS.len()
    }

    fn advance(&mut self, jet: Jet) {
        self.apply_jet(jet);

        if self.can_fall() {
            self.fall();
        } else {
            self.rest();
            self.insert_new_rock(self.next_rock());
        }
    }

    fn fall(&mut self) {
        self.falling_rock_position.y -= 1;
    }

    fn apply_jet(&mut self, jet: Jet) {
        match jet {
            Jet::Left => {
                if self.position_is_free(self.falling_rock_position + Vec2D { x: -1, y: 0 }) {
                    self.falling_rock_position.x -= 1;
                }
            }
            Jet::Right => {
                if self.position_is_free(self.falling_rock_position + Vec2D { x: 1, y: 0 }) {
                    self.falling_rock_position.x += 1;
                }
            }
        }
    }

    fn position_is_free(&self, position: Vec2D<i64>) -> bool {
        // Left wall
        if position.x < 0 {
            return false;
        }

        // Right wall
        if position.x + self.rock().width > CAVE_WIDTH {
            return false;
        }

        // Resting blocks
        self.rock()
            .blocks
            .iter()
            .map(|block_pos| *block_pos + position)
            .all(|block_pos| block_pos.y >= *self.field.get(block_pos.x as usize).unwrap())
    }

    fn can_fall(&self) -> bool {
        // One unit down
        let offset: Vec2D<i64> = Vec2D { x: 0, y: -1 };

        // Bottom floor
        if self.falling_rock_position.y == 0 {
            return false;
        }

        // Other pieces
        self.position_is_free(self.falling_rock_position + offset)
    }

    fn rest(&mut self) {
        // Apply rock to floor shape
        self.rock()
            .blocks
            .iter()
            .map(|b| (*b + self.falling_rock_position))
            .for_each(|pos| {
                self.top = self.top.max(pos.y + 1);
                let current_field = *self.field.get(pos.x as usize).unwrap();
                let new_field = current_field.max(pos.y + 1);
                *self.field.get_mut(pos.x as usize).unwrap() = new_field;
            });
        // Reset lowest point to 0
        self.normalize_field();
        self.resting_rock_count += 1;
    }

    fn normalize_field(&mut self) {
        let lowest_field = *self.field.iter().min().unwrap();
        self.field.iter_mut().for_each(|n| *n -= lowest_field);
        self.top -= lowest_field;
        self.stack_height += lowest_field;
    }

    fn insert_new_rock(&mut self, rock: usize) {
        self.falling_rock = rock;
        self.set_start_position();
    }
}

// https://adventofcode.com/2022/day/17
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let jets: Vec<Jet> = input
        .chars()
        .filter(|c| *c != '\n')
        .map(char::into)
        .collect();

    // unimplemented!();

    let mut p1_tower = RockTower::new(2022, jets.as_slice());
    let mut p2_tower = RockTower::new(1_000_000_000_000, jets.as_slice());

    let tower_height = p1_tower.calc_tower_height();
    // let tower_height_p2 = p2_tower.calc_tower_height();

    let tower_height_p2 = 0;

    Ok(DayOutput {
        part1: Some(PartResult::UInt(tower_height as u64)),
        part2: Some(PartResult::UInt(tower_height_p2 as u64)),
    })
}

// fn count_tower_height(jets: &[Jet], rock_fall_count: i64) -> i64 {}

#[cfg(test)]
mod tests {

    use crate::solutions::day17::RockTower;

    use super::Jet;

    static EXAMPLE_INPUT: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(17, super::solve)
    }

    #[test]
    fn example() {
        let jets: Vec<Jet> = EXAMPLE_INPUT
            .chars()
            .filter(|c| *c != '\n')
            .map(|c| c.into())
            .collect();

        let mut tower = RockTower::new(2022, jets.as_slice());
        tower.inhibit_superblock = true;
        let tower_height = tower.calc_tower_height();

        assert_eq!(tower_height, 3068);
    }

    #[test]
    fn example_superblock() {
        let jets: Vec<Jet> = EXAMPLE_INPUT
            .chars()
            .filter(|c| *c != '\n')
            .map(|c| c.into())
            .collect();

        let mut tower = RockTower::new(2022, jets.as_slice());
        let tower_height = tower.calc_tower_height();

        assert_eq!(tower_height, 3068);
    }

    // /    #[test]
    // fn superblock_parity() {
    //     let jets: Vec<Jet> = EXAMPLE_INPUT
    //         .chars()
    //         .filter(|c| *c != '\n')
    //         .map(|c| c.into())
    //         .collect();

    //     let mut tower = RockTower::new(2022, jets.as_slice());
    //     tower.inhibit_superblock = true;
    //     let real_tower_height = tower.calc_tower_height();

    //     let mut tower2 = RockTower::new(2022, jets.as_slice());
    //     let superblock_tower_height = tower2.calc_tower_height();

    //     assert_eq!(real_tower_height, superblock_tower_height);
    // }
}
