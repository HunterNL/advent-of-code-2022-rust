use std::collections::HashSet;

use crate::grid::Grid;
use crate::vec2d::Vec2DBounds;

use crate::vec2d::{Vec2D, DOWN, LEFT, RIGHT};

use super::{DayOutput, LogicError};

type VecSet = HashSet<Vec2D<i32>>;

const SAND_ENTRY_POINT: Vec2D<i32> = Vec2D { x: 500, y: 0 };

fn insert_line(from: &Vec2D<i32>, to: &Vec2D<i32>, set: &mut HashSet<Vec2D<i32>>) {
    let dir = (*to - *from).normalized();
    let mut cur = *from;
    while cur != *to {
        set.insert(cur);
        cur = cur + dir;
    }
    set.insert(*to);
}

fn build_walls(input: &str) -> HashSet<Vec2D<i32>> {
    let mut walls: HashSet<Vec2D<i32>> = HashSet::new();

    let build_instructions: Vec<Vec<Vec2D<i32>>> = input
        .lines()
        .map(|line| {
            line.split(" -> ")
                // .inspect(|f| println!("{:?}", f))
                .map(|vecstr| vecstr.parse::<Vec2D<i32>>().unwrap())
                .collect()
        })
        .collect();

    build_instructions.iter().for_each(|line| {
        line.windows(2).for_each(|a| {
            if a.len() != 2 {
                panic!("Expected windows of length 2")
            }

            insert_line(&a[0], &a[1], &mut walls);
        })
    });

    walls
}

// Find the lowest point of the given vectors
fn lowest_point(walls: &VecSet) -> i32 {
    walls
        .iter()
        .fold(0, |acc, cur| if (cur.y) > acc { cur.y } else { acc })
}

fn print_cave(cave: &VecSet) {
    let (min, max) = cave.iter().cloned().inspect(|_| {}).bounds_iter();
    let size = max - min;
    let size = size + Vec2D { x: 1, y: 1 };

    let mut content = vec!['_'; (size.x * size.y).try_into().unwrap()];
    content.reserve((size.x * size.y).try_into().unwrap());
    // content.fill_with(|| ' ');

    // let mut grid = Grid::new(size.x.try_into().unwrap(), size.y.try_into().unwrap());
    let mut grid = Grid::new_with_content(content, size.x.try_into().unwrap()).unwrap();

    grid.set(&Vec2D { x: 8, y: 0 }, 'X');
    // println!("{}", grid);

    // println!("size: {:?}", size);

    cave.iter().for_each(|pos| {
        let gridpos = *pos - min;
        // println!("{:?}", gridpos);
        grid.set(&gridpos, 'X');
    });

    println!("{}", grid);
}

fn find_abbys_count(mut walls: VecSet) -> i32 {
    let mut resting_sand_count = 0;
    let floor = lowest_point(&walls);
    let mut sand_pos = SAND_ENTRY_POINT;
    loop {
        let point_below = sand_pos + DOWN;
        let point_below_left = sand_pos + DOWN + LEFT;
        let point_below_right = sand_pos + DOWN + RIGHT;

        if sand_pos.y > floor {
            return resting_sand_count;
        }

        // Nothing below, continue
        if walls.get(&point_below).is_none() {
            sand_pos = point_below;
            continue;
        }

        // Left side free, move there
        if walls.get(&point_below_left).is_none() {
            sand_pos = point_below_left;
            continue;
        }

        // Right side free, move there
        if walls.get(&point_below_right).is_none() {
            sand_pos = point_below_right;
            continue;
        }

        // Nowhere to go, come to rest
        resting_sand_count += 1;

        walls.insert(sand_pos);
        sand_pos = SAND_ENTRY_POINT;
    }
}

fn find_blocked_source_count(mut walls: VecSet) -> i32 {
    let mut resting_sand_count = 0;
    let floor = lowest_point(&walls) + 2;
    let mut sand_pos = SAND_ENTRY_POINT;
    loop {
        let point_below = sand_pos + DOWN;
        let point_below_left = sand_pos + DOWN + LEFT;
        let point_below_right = sand_pos + DOWN + RIGHT;

        if point_below.y == floor {
            resting_sand_count += 1;
            walls.insert(sand_pos);
            sand_pos = SAND_ENTRY_POINT;
            continue;
        }

        // Nothing below, continue
        if walls.get(&point_below).is_none() {
            sand_pos = point_below;
            continue;
        }

        // Left side free, move there
        if walls.get(&point_below_left).is_none() {
            sand_pos = point_below_left;
            continue;
        }

        // Right side free, move there
        if walls.get(&point_below_right).is_none() {
            sand_pos = point_below_right;
            continue;
        }

        // Nowhere to go, come to rest
        resting_sand_count += 1;
        walls.insert(sand_pos);

        // We're blocking the entry, return
        if sand_pos == SAND_ENTRY_POINT {
            return resting_sand_count;
        }

        sand_pos = SAND_ENTRY_POINT;
    }
}

// fn parse_rock_line(s: &str, out: &mut Vec<Vec2D<i32>>) {}

// https://adventofcode.com/2022/day/14
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    // let cave = build_walls(input);
    let abbyscount = find_abbys_count(build_walls(input));
    let source_block_count = find_blocked_source_count(build_walls(input));

    Ok(DayOutput {
        part1: Some(super::PartResult::Int(abbyscount)),
        part2: Some(super::PartResult::Int(source_block_count)),
    })
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, str::FromStr};

    use crate::solutions::day14::print_cave;

    use super::{build_walls, find_abbys_count};

    // use crate::solutions::day13::{decoder_key, sum_indexes, ListItem};

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(14, super::solve)
    }

    #[test]
    fn example() {
        let input = "498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9";
        let cave = build_walls(input);

        // println!("START CAVE");
        // print_cave(&cave);

        let abbyscount = find_abbys_count(cave);

        assert_eq!(abbyscount, 24);
    }
}
