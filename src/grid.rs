use crate::vec2d::Vec2D;

use self::iterators::{EdgeIterator, GridLineIterator};

pub mod iterators;

pub struct Grid<T> {
    bytes: Vec<T>,
    line_size: usize,
}

#[derive(PartialEq, Eq)]
enum Step {
    Top,
    Bottom,
    Left,
    Right,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl<T> Grid<T> {
    // Get a character at the given coordinates
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.bytes.get(x + y * self.line_size)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.bytes.get_mut(x + y * self.line_size)
    }

    pub fn get_by_vec(&self, pos: Vec2D<usize>) -> Option<&T> {
        self.bytes.get(pos.x + pos.y * self.line_size)
    }

    pub fn get_mut_by_vec(&mut self, pos: Vec2D<usize>) -> Option<&mut T> {
        self.bytes.get_mut(pos.x + pos.y * self.line_size)
    }

    pub fn size(&self) -> usize {
        self.line_size
    }

    fn increment_for_direction(&self, dir: Direction) -> i32 {
        match dir {
            Direction::Up => -(self.line_size as i32),
            Direction::Down => self.line_size as i32,
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }

    pub fn edges(&self) -> EdgeIterator<T> {
        EdgeIterator::new(self)
    }

    pub fn line_iter(&self, start: Vec2D<usize>, dir: Direction) -> GridLineIterator<T> {
        let iterations_left = match dir {
            Direction::Up => start.y + 1,
            Direction::Down => self.line_size - start.y,
            Direction::Left => start.x + 1,
            Direction::Right => self.line_size - start.x,
        };

        let increment: i32 = match dir {
            Direction::Up => -(self.line_size as i32),
            Direction::Down => self.line_size as i32,
            Direction::Left => -1,
            Direction::Right => 1,
        };

        GridLineIterator {
            grid: self,
            current: (start.x + start.y * self.line_size) as i32,
            iterations_left,
            increment,
        }
    }
}

impl Grid<u8> {
    pub fn from_str(str: &str) -> Self {
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

// The from trait won't allow the lifetimes needed her
// This doesn't really convert the string, just gathers info on size and does safety checks

#[cfg(test)]
mod tests {

    use crate::{grid::iterators::GridIterator, vec2d::Vec2D};

    use super::*;

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
    fn grid_iter() {
        let mut iter = GridIterator::new(2, 2);

        assert_eq!(iter.next().unwrap(), Vec2D { x: 0, y: 0 });
        assert_eq!(iter.next().unwrap(), Vec2D { x: 1, y: 0 });
        assert_eq!(iter.next().unwrap(), Vec2D { x: 0, y: 1 });
        assert_eq!(iter.next().unwrap(), Vec2D { x: 1, y: 1 });
    }
}
