use core::panic;
use std::{
    collections::HashSet,
    fmt::{Display, Write},
};

use crate::vec2d::Vec2D;

use super::{DayOutput, LogicError, PartResult};

const CAVE_WIDTH: i64 = 7;
const SHAPE_VERTICAL_SPAWN_OFFSET: i64 = 3;
const SHAPE_HORIZONTAL_SPAWN_OFFSET: i64 = 2;

static SHAPE_MINUS: Shape = Shape {
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

static SHAPE_PLUS: Shape = Shape {
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

static SHAPE_L: Shape = Shape {
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

static SHAPE_PIPE: Shape = Shape {
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

static SHAPE_CUBE: Shape = Shape {
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

static SHAPES: [&Shape; 5] = [
    &SHAPE_MINUS,
    &SHAPE_PLUS,
    &SHAPE_L,
    &SHAPE_PIPE,
    &SHAPE_CUBE,
];

struct Shape<'a> {
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
    field: HashSet<Vec2D<i64>>,
    falling_shape: &'static Shape<'static>,
    falling_shape_position: Vec2D<i64>,
    top: i64,
    resting_rock_count: i64,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let top_y = self.falling_shape_position.y + self.falling_shape.height + 1;
        for n in 0..top_y {
            let y = top_y - (n + 1);

            f.write_char('|')?;
            for x in 0..CAVE_WIDTH {
                let charpos = Vec2D { x, y };
                if self.field.contains(&charpos) {
                    f.write_char('#')?;
                } else if self
                    .falling_shape
                    .blocks
                    .iter()
                    .map(|pos| (*pos + self.falling_shape_position))
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
    fn new(start_shape: &'static Shape) -> Self {
        let mut a = Self {
            field: HashSet::new(),
            falling_shape: start_shape,
            falling_shape_position: Vec2D { x: 2, y: 4 },
            top: 0,
            resting_rock_count: 0,
        };
        a.set_start_position();

        a
    }

    fn set_start_position(&mut self) {
        self.falling_shape_position.y = self.top + SHAPE_VERTICAL_SPAWN_OFFSET;
        self.falling_shape_position.x = SHAPE_HORIZONTAL_SPAWN_OFFSET;
    }

    fn advance<'a, 'b>(
        &mut self,
        jet_iter: &mut impl Iterator<Item = &'a Jet>,
        rock_iter: &mut impl Iterator<Item = &'b &'static Shape<'static>>,
    ) {
        self.apply_jet(jet_iter.next().unwrap());

        if self.can_fall() {
            self.fall();
        } else {
            self.rest(rock_iter);
        }
    }

    fn fall(&mut self) {
        self.falling_shape_position.y -= 1;
    }

    fn apply_jet(&mut self, jet: &Jet) {
        match jet {
            Jet::Left => {
                // println!("Jetted left");
                if self.position_is_free(self.falling_shape_position + Vec2D { x: -1, y: 0 }) {
                    self.falling_shape_position.x -= 1;
                }
            }
            Jet::Right => {
                // println!("Jetted right");
                if self.position_is_free(self.falling_shape_position + Vec2D { x: 1, y: 0 }) {
                    self.falling_shape_position.x += 1;
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
        if position.x + self.falling_shape.width > CAVE_WIDTH {
            return false;
        }

        // Resting blocks
        self.falling_shape
            .blocks
            .iter()
            .all(|block_pos| !self.field.contains(&(*block_pos + position)))
    }

    fn can_fall(&self) -> bool {
        let offset: Vec2D<i64> = Vec2D { x: 0, y: -1 };

        if self.falling_shape_position.y == 0 {
            return false;
        }

        self.position_is_free(self.falling_shape_position + offset)
    }

    fn rest<'a>(&mut self, rock_iter: &mut impl Iterator<Item = &'a &'static Shape<'static>>) {
        self.falling_shape
            .blocks
            .iter()
            .map(|b| (*b + self.falling_shape_position))
            .for_each(|pos| {
                self.top = self.top.max(pos.y + 1);
                self.field.insert(pos);
            });

        self.falling_shape = rock_iter.next().unwrap();
        self.set_start_position();
        self.resting_rock_count += 1;
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
    let tower_height_p2 = 0;

    Ok(DayOutput {
        part1: Some(PartResult::UInt(tower_height as u64)),
        part2: Some(PartResult::UInt(tower_height_p2 as u64)),
    })
}

fn count_tower_height(jets: &[Jet], iteration_count: i64) -> i64 {
    let mut jet_iter = jets.iter().cycle();
    let mut rock_iter = SHAPES.iter().cycle();

    let mut state = State::new(rock_iter.next().unwrap());
    let percent = iteration_count / 100;

    loop {
        state.advance(&mut jet_iter, &mut rock_iter);

        if state.resting_rock_count % percent == 0 {
            println!("{}", state.resting_rock_count / percent);
        }

        if state.resting_rock_count == iteration_count {
            break;
        }
    }
    state.top
}

#[cfg(test)]
mod tests {
    use super::{Jet, State, SHAPES};

    static EXAMPLE_INPUT: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

    #[test]
    #[ignore = "wip"]
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

        let mut jet_iter = jets.iter().cycle();
        let mut rock_iter = SHAPES.iter().cycle();

        let mut state = State::new(rock_iter.next().unwrap());

        loop {
            state.advance(&mut jet_iter, &mut rock_iter);

            if state.resting_rock_count == 2022 {
                break;
            }
        }
        assert_eq!(state.top, 3068);
    }
}
