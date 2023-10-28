use std::{collections::HashMap, str::FromStr};

use crate::vec2d::Vec2D;

use super::{DayOutput, LogicError, PartResult};

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn vec_for_dir(dir: &Direction) -> Vec2D<i32> {
    match dir {
        Direction::Up => Vec2D { x: 0, y: -1 },
        Direction::Down => Vec2D { x: 0, y: 1 },
        Direction::Left => Vec2D { x: -1, y: 0 },
        Direction::Right => Vec2D { x: 1, y: 0 },
    }
}

struct Movement {
    direction: Direction,
    distance: i32,
}

#[derive(Default)]
struct Rope {
    head: Vec2D<i32>,
    tail: Vec2D<i32>,
}

#[derive(Default)]
struct RopeLong {
    tail: [Vec2D<i32>; 10],
}

impl RopeSnake for RopeLong {
    fn move_head(&mut self, direction: &Direction) {
        let head = self.tail.get_mut(0).unwrap();
        *head = *head + vec_for_dir(direction);
        let tail_len = self.tail.len();

        for i in 1..tail_len {
            let head = *self.tail.get(i - 1).unwrap();
            let tail = self.tail.get_mut(i).unwrap();

            let dist_to_head = (head - *tail).abs();
            if dist_to_head.x > 1 || dist_to_head.y > 1 {
                update_tail_pos(tail, &head)
            }
        }
    }

    fn get_tail(&self) -> Vec2D<i32> {
        *self.tail.last().unwrap()
    }
}

trait RopeSnake {
    fn move_head(&mut self, direction: &Direction);
    fn get_tail(&self) -> Vec2D<i32>;
}

impl RopeSnake for Rope {
    fn move_head(&mut self, direction: &Direction) {
        self.head = self.head + vec_for_dir(direction);

        let dist_to_head = (self.head - self.tail).abs();
        if dist_to_head.x > 1 || dist_to_head.y > 1 {
            update_tail_pos(&mut self.tail, &self.head)
        }
    }

    fn get_tail(&self) -> Vec2D<i32> {
        self.tail
    }
}

fn update_tail_pos(tail: &mut Vec2D<i32>, head: &Vec2D<i32>) {
    let mut vec_to_head = *head - *tail;
    // Reset both to 1, preserving sign
    // Really hoping the compiler optimizes this instead of idiv
    if vec_to_head.x != 0 {
        vec_to_head.x = vec_to_head.x / vec_to_head.x.abs();
    }

    if vec_to_head.y != 0 {
        vec_to_head.y = vec_to_head.y / vec_to_head.y.abs();
    }

    *tail = *tail + vec_to_head;
}

impl FromStr for Movement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s.split_once(' ').ok_or("Split failed")?;
        let distance: i32 = right.parse().map_err(|_| "Distance parse failed")?;

        Ok(Self {
            direction: match left {
                "U" => Direction::Up,
                "D" => Direction::Down,
                "L" => Direction::Left,
                "R" => Direction::Right,
                &_ => panic!("Unexpected input"),
            },
            distance,
        })
    }
}

// https://adventofcode.com/2022/day/9
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let mut short_rope = Rope::default();
    let mut short_rope_seen_positions: HashMap<Vec2D<i32>, bool> = HashMap::new();

    let mut long_rope = RopeLong::default();
    let mut long_rope_seen_positions: HashMap<Vec2D<i32>, bool> = HashMap::new();

    // short_rope_seen_positions.insert(Vec2D::default(), true);
    // short_rope_seen_positions.insert(Vec2D::default(), true);

    input
        .lines()
        .map(|line| line.parse::<Movement>().unwrap())
        .for_each(|movement| {
            for _ in 0..movement.distance {
                short_rope.move_head(&movement.direction);
                short_rope_seen_positions.insert(short_rope.get_tail(), true);

                long_rope.move_head(&movement.direction);
                long_rope_seen_positions.insert(long_rope.get_tail(), true);
            }
        });

    Ok(DayOutput {
        part1: Some(PartResult::Int(short_rope_seen_positions.len() as i32)),
        part2: Some(PartResult::Int(long_rope_seen_positions.len() as i32)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn day() -> Result<(), String> {
        super::super::tests::test_day(9, super::solve)
    }
}
