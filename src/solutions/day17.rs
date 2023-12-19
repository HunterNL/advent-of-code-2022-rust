use core::panic;
use std::fmt::{Display, Write};

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

enum Jet {
    Left,
    Right,
}

impl From<char> for Jet {
    fn from(value: char) -> Self {
        match value {
            '<' => Self::Left,
            '>' => Self::Right,
            _ => panic!("Unexpected input"),
        }
    }
}

struct State {
    /// Floor shape
    field: [i64; CAVE_WIDTH as usize],

    /// Currently falling rock
    falling_rock: &'static Rock<'static>,

    /// Position of the bottomleft corner of the falling rock
    falling_rock_position: Vec2D<i64>,

    /// Highest point of the floor, used for determining spawn position
    top: i64,

    /// How many rocks have fallen and rested
    resting_rock_count: i64,

    /// Height "below" the floor, added to by normalizing floor shape
    stack_height: i64,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let top_y = self.falling_rock_position.y + self.falling_rock.height + 1;
        for n in 0..top_y {
            let y = top_y - (n + 1);

            f.write_char('|')?;
            for x in 0..CAVE_WIDTH {
                let charpos = Vec2D { x, y };
                if *self.field.get(charpos.x as usize).unwrap() > charpos.y {
                    f.write_char('#')?;
                } else if self
                    .falling_rock
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

impl State {
    fn new(start_rock: &'static Rock) -> Self {
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

    fn set_start_position(&mut self) {
        self.falling_rock_position.y = self.top + ROCK_VERTICAL_SPAWN_OFFSET;
        self.falling_rock_position.x = ROCK_HORIZONTAL_SPAWN_OFFSET;
    }

    fn advance<'a, 'b>(
        &mut self,
        jet: &'a Jet,
        rock_iter: &mut impl Iterator<Item = &'b &'static Rock<'static>>,
    ) {
        self.apply_jet(jet);

        if self.can_fall() {
            self.fall();
        } else {
            self.rest();
            self.insert_new_rock(rock_iter.next().unwrap());
        }
    }

    fn fall(&mut self) {
        self.falling_rock_position.y -= 1;
    }

    fn apply_jet(&mut self, jet: &Jet) {
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
        if position.x + self.falling_rock.width > CAVE_WIDTH {
            return false;
        }

        // Resting blocks
        self.falling_rock
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
        self.falling_rock
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

    fn insert_new_rock(&mut self, rock: &'static Rock<'static>) {
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

    let tower_height = count_tower_height(&jets, 2022);
    // let tower_height_p2 = count_tower_height(&jets, 1_000_000_000_000);
    let tower_height_p2 = 0;

    Ok(DayOutput {
        part1: Some(PartResult::UInt(tower_height as u64)),
        part2: Some(PartResult::UInt(tower_height_p2 as u64)),
    })
}

fn count_tower_height(jets: &[Jet], rock_fall_count: i64) -> i64 {
    let mut jet_iter = jets.iter().cycle();
    let mut rock_iter = ROCKS.iter().cycle();

    let mut state = State::new(rock_iter.next().unwrap());
    // let percent = iteration_count / 100;

    while state.resting_rock_count < rock_fall_count {
        state.advance(jet_iter.next().unwrap(), &mut rock_iter);
    }
    state.top + state.stack_height
}

#[cfg(test)]
mod tests {

    use crate::solutions::day17::count_tower_height;

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

        let count = count_tower_height(jets.as_slice(), 2022);
        assert_eq!(count, 3068);
    }
}
