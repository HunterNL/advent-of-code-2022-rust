use std::collections::HashMap;

use crate::vec2d::Vec2D;

use super::{DayOutput, LogicError, PartResult};

const TALLEST_TREE: u8 = b'9';

struct CharacterGrid {
    bytes: Vec<u8>,
    line_size: usize,
}

#[derive(PartialEq, Eq)]
enum Step {
    Top,
    Bottom,
    Left,
    Right,
}

// Iterates over every edge of the grid, emitting GridLineIterators
struct EdgeIterator<'a> {
    grid: &'a CharacterGrid,
    step: Step,
    index: usize,
    iterations_left: usize,
}

impl<'a> Iterator for EdgeIterator<'a> {
    type Item = GridLineIterator<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Increment = how to get to the next edge
        // Top and bottom advance by one, left and right increment a whole line
        let line_size = self.grid.line_size;
        let increment = match self.step {
            Step::Top | Step::Bottom => 1,
            Step::Left | Step::Right => self.grid.line_size,
        };

        // Peek direction is how the inner iterator advances, it is orthagonal to self.increment
        let peek_direction = match self.step {
            Step::Top => self.grid.increment_for_direction(Direction::Down),
            Step::Bottom => self.grid.increment_for_direction(Direction::Up),
            Step::Left => self.grid.increment_for_direction(Direction::Right),
            Step::Right => self.grid.increment_for_direction(Direction::Left),
        };

        let out = GridLineIterator {
            grid: self.grid,
            current: self.index as i32,
            iterations_left: line_size,
            increment: peek_direction,
        };

        self.index += increment;
        self.iterations_left -= 1;

        // If we've reached the end of an edge, switch to the next edge or stop
        if self.iterations_left == 0 {
            self.iterations_left = line_size;
            self.index = 0;
            (self.step, self.index) = match self.step {
                Step::Top => (Step::Bottom, line_size * line_size - line_size),
                Step::Bottom => (Step::Left, 0),
                Step::Left => (Step::Right, line_size - 1),
                Step::Right => (Step::Top, 0),
            };

            if self.step == Step::Top {
                return None;
            }
        }
        Some(out)
    }
}

// Iterates in a straight line over a grid
struct GridLineIterator<'a> {
    grid: &'a CharacterGrid,
    current: i32,
    iterations_left: usize,
    increment: i32,
}

impl<'a> Iterator for GridLineIterator<'a> {
    type Item = (i32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        // println!("{}", self.current);
        self.current += self.increment;
        if self.iterations_left == 0 {
            // print!("PeekIterator Done!");
            return None;
        }
        self.iterations_left -= 1;
        self.grid
            .bytes
            .get((self.current - self.increment) as usize)
            .map(|u| (self.current - self.increment, *u))
    }
}

struct SightlineIterator<'a> {
    iter: GridLineIterator<'a>,
    max_height: i32,
}

impl<'a> Iterator for SightlineIterator<'a> {
    type Item = (i32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(entry) => {
                if entry.1 as i32 >= self.max_height {
                    None
                } else {
                    Some(entry)
                }
            }
            None => None,
        }
    }
}

struct VisableTreeIterator<'a> {
    iter: GridLineIterator<'a>,
    seen_first: bool,
    highest_seen: i32,
}

impl<'a> VisableTreeIterator<'a> {
    fn new(iter: GridLineIterator) -> VisableTreeIterator<'_> {
        VisableTreeIterator {
            iter,
            seen_first: false,
            highest_seen: -1,
        }
    }
}

impl<'a> Iterator for VisableTreeIterator<'a> {
    type Item = (i32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.seen_first {
            return match self.iter.next() {
                Some(entry) => {
                    self.seen_first = true;
                    self.highest_seen = entry.1 as i32;
                    Some(entry)
                }
                None => None,
            };
        }

        if self.highest_seen == TALLEST_TREE as i32 {
            return None;
        }

        for entry in self.iter.by_ref() {
            let tree_height = entry.1 as i32;

            if tree_height > self.highest_seen {
                self.highest_seen = tree_height;
                return Some(entry);
            }
        }

        None
    }
}

// Iterates over a grid, row by row
struct GridIterator {
    pos: Vec2D<usize>,
    max: Vec2D<usize>,
}

impl GridIterator {
    fn new(width: usize, height: usize) -> Self {
        Self {
            pos: Vec2D { x: 0, y: 0 },
            max: Vec2D {
                x: width,
                y: height,
            },
        }
    }
}

impl Iterator for GridIterator {
    type Item = Vec2D<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        // Always capture the current state to output
        if self.pos.y < self.max.y {
            let current = self.pos;
            self.pos.x += 1;

            if self.pos.x == self.max.x {
                self.pos.x = 0;
                self.pos.y += 1;
            }

            Some(current)
        } else {
            None
        }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl CharacterGrid {
    // Get a character at the given coordinates
    fn get(&self, x: usize, y: usize) -> Option<u8> {
        self.bytes.get(x + y * self.line_size).copied()
    }

    fn increment_for_direction(&self, dir: Direction) -> i32 {
        match dir {
            Direction::Up => -(self.line_size as i32),
            Direction::Down => self.line_size as i32,
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }

    fn sideline_peek_iters(&self) -> EdgeIterator {
        EdgeIterator {
            grid: self,
            step: Step::Top,
            index: 0,
            iterations_left: self.line_size,
        }
    }

    // The from trait won't allow the lifetimes needed her
    // This doesn't really convert the string, just gathers info on size and does safety checks
    fn from_str(str: &str) -> Self {
        //1: Ensure all lines have the same length
        let size = str.lines().next().unwrap().bytes().len();
        let mut v: Vec<u8> = Vec::new();
        v.reserve(size * size);

        let equal_sizes = str.lines().all(|line| line.bytes().len() == size);
        if !equal_sizes {
            println!("The following line lenghts were seen");
            str.lines()
                .map(|line| line.bytes().len())
                .for_each(|line_len| println!("{line_len}"));

            panic!("Line lenghts don't match");
        }

        str.lines()
            .map(|line| line.bytes())
            .for_each(|f| v.extend(f));

        Self {
            bytes: v,
            line_size: size,
        }
    }
}

fn find_treehouse_spot(grid: &CharacterGrid) -> i32 {
    GridIterator::new(grid.line_size, grid.line_size)
        .map(|position| score_treehouse_spot(grid, position))
        // .inspect(|f| println!("{f}"))
        .max()
        .expect("number")
}

fn score_treehouse_spot(grid: &CharacterGrid, position: Vec2D<usize>) -> i32 {
    let line_size = grid.line_size as i32;
    let tree_size = grid.get(position.x, position.y).unwrap();

    let top_sightline_count =
        count_visible_trees(grid, position, -line_size, position.y + 1, tree_size);

    let bottom_sightline_count = count_visible_trees(
        grid,
        position,
        line_size,
        (line_size - position.y as i32) as usize,
        tree_size,
    );

    let right_sightline_count = count_visible_trees(
        grid,
        position,
        1,
        (line_size - position.x as i32) as usize,
        tree_size,
    );

    let left_sightline_count = count_visible_trees(
        grid,
        position,
        -1,
        (position.x as i32 + 1) as usize,
        tree_size,
    );

    top_sightline_count * right_sightline_count * bottom_sightline_count * left_sightline_count
}

fn count_visible_trees(
    grid: &CharacterGrid,
    position: Vec2D<usize>,
    increment: i32,
    max_iters: usize,
    max_tree_size: u8,
) -> i32 {
    let mut a = GridLineIterator {
        grid,
        current: (position.x + position.y * grid.line_size) as i32,
        iterations_left: max_iters,
        increment,
    };

    a.next(); // Skip the starting tile, it'd instantly stop at the start tree

    let mut count = 0;

    for entry in a {
        count += 1;
        let tree_height = entry.1;

        if tree_height >= max_tree_size {
            break;
        }
    }

    count
}

fn count_trees(grid: &CharacterGrid) -> i32 {
    let mut seen_trees = HashMap::new();

    for peek in grid.sideline_peek_iters() {
        VisableTreeIterator::new(peek).for_each(|tree| {
            seen_trees.insert(tree.0, true);
        })
    }

    seen_trees.len() as i32
}

// https://adventofcode.com/2022/day/8
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let grid = CharacterGrid::from_str(input);

    let seen_tree_count = count_trees(&grid);
    let treehouse_score = find_treehouse_spot(&grid);

    Ok(DayOutput {
        part1: Some(PartResult::Int(seen_tree_count)),
        part2: Some(PartResult::Int(treehouse_score)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(8, super::solve)
    }

    #[test]
    fn grid_edge_iter() -> Result<(), String> {
        #[rustfmt::skip]
        let input = [
            "30373", 
            "25512", 
            "65332", 
            "33549", 
            "35390"].join("\n");

        let grid = CharacterGrid::from_str(&input);
        let mut iter = grid.sideline_peek_iters();

        // First vertical
        assert_eq!(
            vec![b'3', b'2', b'6', b'3', b'3'],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Second vertical
        assert_eq!(
            vec![b'0', b'5', b'5', b'3', b'5'],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Skip the next 3 verticals
        iter.next();
        iter.next();
        iter.next();

        //First from the bottom row
        assert_eq!(
            vec![b'3', b'3', b'6', b'2', b'3'],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        //Second on bottom row
        assert_eq!(
            vec![b'5', b'3', b'5', b'5', b'0'],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Skip the next 3 verticals
        iter.next();
        iter.next();
        iter.next();

        // First horizontal
        assert_eq!(
            vec![b'3', b'0', b'3', b'7', b'3'],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Second horizontal
        assert_eq!(
            vec![b'2', b'5', b'5', b'1', b'2'],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Skip the next 3 horizontals
        iter.next();
        iter.next();
        iter.next();

        // First horizontal from the right
        assert_eq!(
            vec![b'3', b'7', b'3', b'0', b'3'],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Second horizontal from the right
        assert_eq!(
            vec![b'2', b'1', b'5', b'5', b'2'],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        Ok(())
    }

    #[test]
    fn tree_count() {
        #[rustfmt::skip]
        let input = [
            "30373", 
            "25512", 
            "65332", 
            "33549", 
            "35390"].join("\n");

        let grid = CharacterGrid::from_str(&input);

        assert_eq!(count_trees(&grid), 21);
    }

    #[test]
    fn treehouse_score_single_a() {
        #[rustfmt::skip]
        let input = [
            "30373", 
            "25512", 
            "65332", 
            "33549", 
            "35390"].join("\n");

        let grid = CharacterGrid::from_str(&input);

        assert_eq!(score_treehouse_spot(&grid, Vec2D { x: 2, y: 3 }), 8);
    }

    #[test]
    fn treehouse_score_single_b() {
        #[rustfmt::skip]
        let input = [
            "30373", 
            "25512", 
            "65332", 
            "33549", 
            "35390"].join("\n");

        let grid = CharacterGrid::from_str(&input);

        assert_eq!(score_treehouse_spot(&grid, Vec2D { x: 2, y: 1 }), 4);
    }

    #[test]
    fn treehouse_find() {
        #[rustfmt::skip]
        let input = [
            "30373", 
            "25512", 
            "65332", 
            "33549", 
            "35390"].join("\n");

        let grid = CharacterGrid::from_str(&input);
        let score = find_treehouse_spot(&grid);

        assert_eq!(score, 8);
    }

    #[test]
    fn grid_iter() {
        let mut iter = GridIterator {
            pos: Vec2D { x: 0, y: 0 },
            max: Vec2D { x: 2, y: 2 },
        };

        assert_eq!(iter.next().unwrap(), Vec2D { x: 0, y: 0 });
        assert_eq!(iter.next().unwrap(), Vec2D { x: 1, y: 0 });
        assert_eq!(iter.next().unwrap(), Vec2D { x: 0, y: 1 });
        assert_eq!(iter.next().unwrap(), Vec2D { x: 1, y: 1 });
    }
}
