use std::{cell::Cell, cmp, collections::HashMap};

use super::{DayOutput, LogicError, PartResult};

const TALLEST_TREE: u8 = 9;

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

// Iterates over every edge of the grid, emitting PeekIterators

struct SidelineIterator<'a> {
    grid: &'a CharacterGrid,
    step: Step,
    index: usize,
    iterations_left: usize,
}

impl<'a> Iterator for SidelineIterator<'a> {
    type Item = PeekIterator<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Increment = how to get to the next edge
        // Top and bottom advance by one, left and right increment a whole line
        let line_size = self.grid.line_size;
        let increment = match self.step {
            Step::Top => 1,
            Step::Bottom => 1,
            Step::Left => self.grid.line_size,
            Step::Right => self.grid.line_size,
        };

        // Peek direction is how the inner iterator advances, it is orthagonal to self.increment
        let peek_direction = match self.step {
            Step::Top => line_size as i32,
            Step::Bottom => -(line_size as i32),
            Step::Left => 1,
            Step::Right => -1,
        };

        let out = PeekIterator {
            grid: self.grid,
            current: self.index as i32,
            iterations_left: line_size,
            increment: peek_direction,
        };

        self.index += increment;
        self.iterations_left -= 1;

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

// Iterates inward from an edge
struct PeekIterator<'a> {
    grid: &'a CharacterGrid,
    current: i32,
    iterations_left: usize,
    increment: i32,
}

impl<'a> Iterator for PeekIterator<'a> {
    type Item = (i32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        self.current += self.increment;
        if self.iterations_left == 0 {
            return None;
        }
        self.iterations_left -= 1;
        self.grid
            .bytes
            .get((self.current - self.increment) as usize)
            .map(|u| (self.current - self.increment, *u - ('0' as u8)))
    }
}

struct VisableTreeIterator<'a> {
    iter: PeekIterator<'a>,
    seen_first: bool,
    highest_seen: u8,
}

impl<'a> VisableTreeIterator<'a> {
    fn new(iter: PeekIterator) -> VisableTreeIterator<'_> {
        VisableTreeIterator {
            iter,
            seen_first: false,
            highest_seen: 0,
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
                    self.highest_seen = entry.1;
                    Some(entry)
                }
                None => None,
            };
        }

        if self.highest_seen == TALLEST_TREE {
            return None;
        }

        while let Some(entry) = self.iter.next() {
            let tree_height = entry.1;

            if tree_height > self.highest_seen {
                self.highest_seen = tree_height;
                return Some(entry);
            }
        }

        None
    }
}

impl CharacterGrid {
    // Get a character at the given coordinates
    fn get(&self, x: usize, y: usize) -> u8 {
        return *self.bytes.get(x + y * self.line_size).unwrap();
    }

    fn sideline_peek_iters(&self) -> SidelineIterator {
        SidelineIterator {
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

        //2: Ensure all newlines have the same length

        CharacterGrid {
            bytes: v,
            line_size: size,
        }
    }
}

// impl<'a> FromString<&str> for CharacterGrid<'a> {}

// struct Scanline {
//     start: usize,
//     scanline_advance: i8,
//     scanline_vision_advance: i8,
// }

fn count_trees(grid: &CharacterGrid) -> i32 {
    let mut seen_trees = HashMap::new();

    for peek in grid.sideline_peek_iters() {
        VisableTreeIterator::new(peek).for_each(|tree| {
            // println!("Seen tree {} with height {}", tree.0, tree.1);
            seen_trees.insert(tree.0, true);
        })
    }

    // for mut peek in grid.sideline_peek_iters() {
    //     // println!("\n{}", "=".repeat(10));

    //     let first = peek.next().unwrap();
    //     let mut highest_tree_seen = first.1;
    //     seen_trees.insert(first.0, true);

    //     loop {
    //         let cur = peek.next();
    //         if cur.is_none() {
    //             break;
    //         }

    //         let (index, tree_height) = cur.unwrap();

    //         if tree_height > highest_tree_seen {
    //             seen_trees.insert(index, true);
    //             highest_tree_seen = tree_height;

    //             if highest_tree_seen == TALLEST_TREE {
    //                 break; // No point, there wont' be higher trees to see
    //             }
    //         }
    //     }
    // }

    seen_trees.len() as i32
}

// https://adventofcode.com/2022/day/8
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let grid = CharacterGrid::from_str(input);

    // let mut seen_trees = HashMap::new();
    // for peek in grid.sideline_peek_iters() {
    //     let current_tree = Cell::new(0);
    //     peek.take_while(|entry| entry.1 > current_tree.get())
    //         .for_each(|entry| {
    //             current_tree.set(cmp::max(current_tree.get(), entry.1));
    //             seen_trees.insert(entry.0, true);
    //         })
    // }

    let seen_tree_count = count_trees(&grid);

    // let scanlines: Vec<Scanline> = vec![Scanline {
    //     start: 0,
    //     scanline_advance: 1,
    //     scanline_vision_advance: grid.line_size as i8,
    // }];

    // scanlines.iter().for_each(|scanline| {
    //     let mut highest_seen_tree = 0;
    //     for i in 0..grid.line_size {
    //         let cur = grid.bytes.get(index)
    //     }
    // });

    Ok(DayOutput {
        part1: Some(PartResult::Int(seen_tree_count)),
        part2: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(8, super::solve)
    }

    #[test]
    fn peek_iter() -> Result<(), String> {
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
            vec![3, 2, 6, 3, 3],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Second vertical
        assert_eq!(
            vec![0, 5, 5, 3, 5],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Skip the next 3 verticals
        iter.next();
        iter.next();
        iter.next();

        //First from the bottom row
        assert_eq!(
            vec![3, 3, 6, 2, 3],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        //Second on bottom row
        assert_eq!(
            vec![5, 3, 5, 5, 0],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Skip the next 3 verticals
        iter.next();
        iter.next();
        iter.next();

        // First horizontal
        assert_eq!(
            vec![3, 0, 3, 7, 3],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Second horizontal
        assert_eq!(
            vec![2, 5, 5, 1, 2],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Skip the next 3 horizontals
        iter.next();
        iter.next();
        iter.next();

        // First horizontal from the right
        assert_eq!(
            vec![3, 7, 3, 0, 3],
            iter.next().unwrap().map(|a| a.1).collect::<Vec<u8>>()
        );

        // Second horizontal from the right
        assert_eq!(
            vec![2, 1, 5, 5, 2],
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
}
