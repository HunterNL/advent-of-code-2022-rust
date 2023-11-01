use std::collections::HashMap;

use crate::grid::iterators::GridIterator;
use crate::grid::iterators::GridLineIterator;
use crate::grid::Direction;
use crate::grid::Grid;
use crate::vec2d::Vec2D;

use super::{DayOutput, LogicError, PartResult};

const TALLEST_TREE: u8 = b'9';

struct SightlineIterator<'a> {
    iter: GridLineIterator<'a, u8>,
    max_height: i32,
}

impl<'a> Iterator for SightlineIterator<'a> {
    type Item = (i32, &'a u8);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(entry) => {
                if i32::from(*entry.1) >= self.max_height {
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
    iter: GridLineIterator<'a, u8>,
    seen_first: bool,
    highest_seen: i32,
}

impl<'a> VisableTreeIterator<'a> {
    fn new(iter: GridLineIterator<u8>) -> VisableTreeIterator<'_> {
        VisableTreeIterator {
            iter,
            seen_first: false,
            highest_seen: -1,
        }
    }
}

impl<'a> Iterator for VisableTreeIterator<'a> {
    type Item = (i32, &'a u8);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.seen_first {
            return match self.iter.next() {
                Some(entry) => {
                    self.seen_first = true;
                    self.highest_seen = i32::from(*entry.1);
                    Some(entry)
                }
                None => None,
            };
        }

        if self.highest_seen == i32::from(TALLEST_TREE) {
            return None;
        }

        for entry in self.iter.by_ref() {
            let tree_height = i32::from(*entry.1);

            if tree_height > self.highest_seen {
                self.highest_seen = tree_height;
                return Some(entry);
            }
        }

        None
    }
}

fn find_treehouse_spot(grid: &Grid<u8>) -> i32 {
    GridIterator::new(grid.size(), grid.size())
        .map(|position| score_treehouse_spot(grid, position))
        // .inspect(|f| println!("{f}"))
        .max()
        .expect("number")
}

fn score_treehouse_spot(grid: &Grid<u8>, position: Vec2D<usize>) -> i32 {
    let top_sightline_count = count_visible_trees(grid, position, Direction::Up);
    let bottom_sightline_count = count_visible_trees(grid, position, Direction::Down);
    let right_sightline_count = count_visible_trees(grid, position, Direction::Right);
    let left_sightline_count = count_visible_trees(grid, position, Direction::Left);

    top_sightline_count * right_sightline_count * bottom_sightline_count * left_sightline_count
}

fn count_visible_trees(grid: &Grid<u8>, position: Vec2D<usize>, dir: Direction) -> i32 {
    let mut a = grid.line_iter(position, dir);

    let max_tree_size = *a.next().unwrap().1; // Skip the starting tile and use it as height cap

    let mut count = 0;

    for entry in a {
        count += 1;
        let tree_height = *entry.1;

        if tree_height >= max_tree_size {
            break;
        }
    }

    count
}

fn count_trees(grid: &Grid<u8>) -> i32 {
    let mut seen_trees = HashMap::new();

    for peek in grid.edges() {
        VisableTreeIterator::new(peek).for_each(|tree| {
            seen_trees.insert(tree.0, true);
        });
    }

    seen_trees.len() as i32
}

// https://adventofcode.com/2022/day/8
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let grid = Grid::from_str(input);

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

        let grid = Grid::from_str(&input);
        let mut iter = grid.edges();

        // First vertical
        assert_eq!(
            vec![b'3', b'2', b'6', b'3', b'3'],
            iter.next()
                .unwrap()
                .map(|a| *a.1)
                .collect::<Vec<u8>>()
        );

        // Second vertical
        assert_eq!(
            vec![b'0', b'5', b'5', b'3', b'5'],
            iter.next()
                .unwrap()
                .map(|a| *a.1)
                .collect::<Vec<u8>>()
        );

        // Skip the next 3 verticals
        iter.next();
        iter.next();
        iter.next();

        //First from the bottom row
        assert_eq!(
            vec![b'3', b'3', b'6', b'2', b'3'],
            iter.next()
                .unwrap()
                .map(|a| *a.1)
                .collect::<Vec<u8>>()
        );

        //Second on bottom row
        assert_eq!(
            vec![b'5', b'3', b'5', b'5', b'0'],
            iter.next()
                .unwrap()
                .map(|a| *a.1)
                .collect::<Vec<u8>>()
        );

        // Skip the next 3 verticals
        iter.next();
        iter.next();
        iter.next();

        // First horizontal
        assert_eq!(
            vec![b'3', b'0', b'3', b'7', b'3'],
            iter.next()
                .unwrap()
                .map(|a| *a.1)
                .collect::<Vec<u8>>()
        );

        // Second horizontal
        assert_eq!(
            vec![b'2', b'5', b'5', b'1', b'2'],
            iter.next()
                .unwrap()
                .map(|a| *a.1)
                .collect::<Vec<u8>>()
        );

        // Skip the next 3 horizontals
        iter.next();
        iter.next();
        iter.next();

        // First horizontal from the right
        assert_eq!(
            vec![b'3', b'7', b'3', b'0', b'3'],
            iter.next()
                .unwrap()
                .map(|a| *a.1)
                .collect::<Vec<u8>>()
        );

        // Second horizontal from the right
        assert_eq!(
            vec![b'2', b'1', b'5', b'5', b'2'],
            iter.next()
                .unwrap()
                .map(|a| *a.1)
                .collect::<Vec<u8>>()
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

        let grid = Grid::from_str(&input);

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

        let grid = Grid::from_str(&input);

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

        let grid = Grid::from_str(&input);

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

        let grid = Grid::from_str(&input);
        let score = find_treehouse_spot(&grid);

        assert_eq!(score, 8);
    }

    #[test]
    fn grid_iter() {
        let mut iter = GridIterator::new(2, 2);

        assert_eq!(iter.next().unwrap(), Vec2D { x: 0, y: 0 });
        assert_eq!(iter.next().unwrap(), Vec2D { x: 1, y: 0 });
        assert_eq!(iter.next().unwrap(), Vec2D { x: 0, y: 1 });
        assert_eq!(iter.next().unwrap(), Vec2D { x: 1, y: 1 });
    }
}
