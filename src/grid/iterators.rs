use crate::vec2d::Vec2D;

use super::{Direction, Grid, Step};

// Iterates over every edge of the grid, emitting GridLineIterators
pub struct EdgeIterator<'a, T> {
    grid: &'a Grid<T>,
    step: Step,
    index: usize,
    iterations_left: usize,
}

impl<'a, T> EdgeIterator<'a, T> {
    pub(crate) fn new(grid: &'a Grid<T>) -> EdgeIterator<'a, T> {
        EdgeIterator {
            grid,
            step: Step::Top,
            index: 0,
            iterations_left: grid.height,
        }
    }
}

impl<'a, T> Iterator for EdgeIterator<'a, T> {
    type Item = GridLineIterator<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Increment = how to get to the next edge
        // Top and bottom advance by one, left and right increment a whole line
        let line_size = self.grid.width;
        let increment = match self.step {
            Step::Top | Step::Bottom => 1,
            Step::Left | Step::Right => line_size,
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
pub struct GridLineIterator<'a, T> {
    pub(super) grid: &'a Grid<T>,
    pub(super) current: i32,
    pub(super) iterations_left: usize,
    pub(super) increment: i32,
}

impl<'a, T> Iterator for GridLineIterator<'a, T> {
    type Item = (i32, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.current += self.increment;
        if self.iterations_left == 0 {
            return None;
        }
        self.iterations_left -= 1;
        self.grid
            .bytes
            .get((self.current - self.increment) as usize)
            .map(|u| (self.current - self.increment, u))
    }
}

// Iterates over a grid, row by row
pub struct GridIterator {
    pos: Vec2D<usize>,
    max: Vec2D<usize>,
}

impl GridIterator {
    pub fn new(width: usize, height: usize) -> Self {
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
