use std::{collections::HashSet, str::FromStr};

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

trait RopeSnake {
    fn move_head(&mut self, direction: &Direction);
    fn get_tail(&self) -> Vec2D<i32>;
}

impl RopeSnake for [Vec2D<i32>] {
    fn move_head(&mut self, direction: &Direction) {
        let head = self
            .first_mut()
            .expect("Array to have at least 1 item (should have 2 later in this function)");
        *head = *head + vec_for_dir(direction);
        let tail_len = self.len();

        for i in 1..tail_len {
            let head = *self
                .get(i - 1)
                .expect("Previous item to be available (loop should skip head)");
            let tail = self.get_mut(i).expect("array[i] to be available");

            let dist_to_head = (head - *tail).abs();
            if dist_to_head.x > 1 || dist_to_head.y > 1 {
                update_tail_pos(tail, &head);
            }
        }
    }

    fn get_tail(&self) -> Vec2D<i32> {
        *self.last().expect("Array to have at least 1 item")
    }
}

fn sign(x: i32) -> i32 {
    match x.cmp(&0) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn update_tail_pos(tail: &mut Vec2D<i32>, head: &Vec2D<i32>) {
    let mut vec_to_head = *head - *tail;

    vec_to_head.x = sign(vec_to_head.x);
    vec_to_head.y = sign(vec_to_head.y);

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
    let mut short_rope = [Vec2D::default(); 2];
    let mut short_rope_seen_positions: HashSet<Vec2D<i32>> = HashSet::new();

    let mut long_rope = [Vec2D::default(); 10];
    let mut long_rope_seen_positions: HashSet<Vec2D<i32>> = HashSet::new();

    input
        .lines()
        .map(|line| line.parse::<Movement>().unwrap())
        .for_each(|movement| {
            for _ in 0..movement.distance {
                short_rope.move_head(&movement.direction);
                short_rope_seen_positions.insert(short_rope.get_tail());

                long_rope.move_head(&movement.direction);
                long_rope_seen_positions.insert(long_rope.get_tail());
            }
        });

    Ok(DayOutput {
        part1: Some(PartResult::Int(short_rope_seen_positions.len() as i32)),
        part2: Some(PartResult::Int(long_rope_seen_positions.len() as i32)),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(9, super::solve)
    }
}
