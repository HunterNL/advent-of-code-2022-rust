use std::fmt::{Display, Write};

use crate::vec2d::{self, Vec2D};

use self::iterators::{EdgeIterator, GridIterator, GridLineIterator};

pub mod iterators;

pub struct GridContentIterator<'a, T> {
    grid: &'a Grid<T>,
    index: usize,
}

impl<'a, T> Iterator for GridContentIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.grid.bytes.get(self.index - 1)
    }
}

pub struct GridContentMutIterator<'a, T> {
    grid: &'a mut Grid<T>,
    index: usize,
}

// impl<'a, T> Iterator for GridContentMutIterator<'a, T> {
//     type Item = &'a mut T;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.index += 1;
//         self.grid.bytes.get_mut(self.index - 1).map(move |b| b)
//     }
// }

pub struct Grid<T> {
    bytes: Vec<T>,
    width: usize,
    height: usize,
}

// impl<T> Display for Grid<T>
// where
//     T: Display,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.bytes
//             .chunks(self.width)
//             .try_for_each(|chunk| -> std::fmt::Result {
//                 chunk.iter().try_for_each(|c| -> std::fmt::Result {
//                     f.write_fmt(format_args!("{:3}", c))?;

//                     Ok(())
//                 })?;

//                 f.write_char('\n')?;

//                 Ok(())
//             })?;

//         Ok(())
//     }
// }

impl Display for Grid<u8> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bytes
            .chunks(self.width)
            .try_for_each(|chunk| -> std::fmt::Result {
                chunk.iter().try_for_each(|c| -> std::fmt::Result {
                    f.write_fmt(format_args!("{}", *c as char))?;

                    Ok(())
                })?;

                f.write_char('\n')?;

                Ok(())
            })?;

        Ok(())
    }
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
        self.bytes.get(x + y * self.width)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.bytes.get_mut(x + y * self.width)
    }

    pub fn get_by_vec(&self, pos: &Vec2D<i32>) -> Option<&T> {
        // println!("{} {}", pos.x, pos.y);
        self.bytes.get(pos.x as usize + pos.y as usize * self.width)
    }

    pub fn get_mut_by_vec(&mut self, pos: Vec2D<usize>) -> Option<&mut T> {
        self.bytes.get_mut(pos.x + pos.y * self.width)
    }

    pub fn set(&mut self, pos: &Vec2D<i32>, i: T) {
        let index = self.index_of_position(pos);
        *self.bytes.get_mut(index).unwrap() = i
    }

    // pub fn size(&self) -> usize {
    //     self.width * self.height
    // }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn position_of_index(&self, index: usize) -> Option<Vec2D<i32>> {
        let y = index / self.width;
        let x = index % self.width;

        Some(Vec2D {
            x: x.try_into().unwrap(),
            y: y.try_into().unwrap(),
        })
    }

    pub fn index_of_position(&self, position: &Vec2D<i32>) -> usize {
        return position.x as usize + position.y as usize * self.width as usize;
    }

    pub fn iter(&self) -> GridContentIterator<T> {
        GridContentIterator {
            grid: self,
            index: 0,
        }
    }
    pub fn get_neighbours(&self, pos: Vec2D<i32>, v: &mut Vec<Vec2D<i32>>) {
        let (x, y) = (pos.x, pos.y);

        //Left
        if x > 0 {
            v.push(Vec2D { x: x - 1, y });
        }

        //Right
        if x < self.width as i32 - 1 {
            v.push(Vec2D { x: x + 1, y });
        }

        // Top
        if y > 0 {
            v.push(Vec2D { x, y: y - 1 })
        }

        // Bottom
        if y < self.height as i32 - 1 {
            v.push(Vec2D { x, y: y + 1 })
        }
    }

    pub fn get_neighbours_diagonal(&self, pos: Vec2D<i32>, v: &mut Vec<Vec2D<i32>>) {
        let (x, y) = (pos.x, pos.y);

        if x > 0 {
            //Left side exists

            if y > 0 {
                // Topleft

                v.push(Vec2D { x: x - 1, y: y - 1 })
            }

            //Left
            v.push(Vec2D { x: x - 1, y });

            if y < self.height as i32 - 1 {
                // Bottomleft
                v.push(Vec2D { x: x - 1, y: y + 1 })
            }
        }

        if x < self.width as i32 - 1 {
            // Right side exsists

            if y > 0 {
                // Topright

                v.push(Vec2D { x: x + 1, y: y - 1 })
            }

            //Left
            v.push(Vec2D { x: x + 1, y });

            if y < self.height as i32 - 1 {
                // Bottomright
                v.push(Vec2D { x: x + 1, y: y + 1 })
            }
        }

        // Center top
        if y > 0 {
            v.push(Vec2D { x, y: y - 1 })
        }

        // Center bottom
        if y < self.height as i32 - 1 {
            v.push(Vec2D { x, y: y + 1 })
        }
    }

    fn increment_for_direction(&self, dir: Direction) -> i32 {
        match dir {
            Direction::Up => -(self.width as i32),
            Direction::Down => self.width as i32,
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
            Direction::Down => self.width - start.y,
            Direction::Left => start.x + 1,
            Direction::Right => self.width - start.x,
        };

        let increment: i32 = match dir {
            Direction::Up => -(self.width as i32),
            Direction::Down => self.width as i32,
            Direction::Left => -1,
            Direction::Right => 1,
        };

        GridLineIterator {
            grid: self,
            current: (start.x + start.y * self.width) as i32,
            iterations_left,
            increment,
        }
    }

    pub fn iter_with_pos(&mut self) -> impl Iterator<Item = (Vec2D<usize>, &T)> {
        let grid_iterator = GridIterator::new(self.width, self.height);
        grid_iterator.zip(self.bytes.iter())
    }

    pub fn iter_mut_with_pos(&mut self) -> impl Iterator<Item = (Vec2D<usize>, &mut T)> {
        let grid_iterator = GridIterator::new(self.width, self.height);
        grid_iterator.zip(self.bytes.iter_mut())
        // self.bytes.iter_mut().zip(GridIterator)
    }

    pub fn take(self) -> Vec<T> {
        return self.bytes;
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
            width: size,
            height: str.lines().count(),
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
            iter.next().unwrap().map(|a| *a.1).collect::<Vec<u8>>()
        );

        // Second vertical
        assert_eq!(
            vec![b'0', b'5', b'5', b'3', b'5'],
            iter.next().unwrap().map(|a| *a.1).collect::<Vec<u8>>()
        );

        // Skip the next 3 verticals
        iter.next();
        iter.next();
        iter.next();

        //First from the bottom row
        assert_eq!(
            vec![b'3', b'3', b'6', b'2', b'3'],
            iter.next().unwrap().map(|a| *a.1).collect::<Vec<u8>>()
        );

        //Second on bottom row
        assert_eq!(
            vec![b'5', b'3', b'5', b'5', b'0'],
            iter.next().unwrap().map(|a| *a.1).collect::<Vec<u8>>()
        );

        // Skip the next 3 verticals
        iter.next();
        iter.next();
        iter.next();

        // First horizontal
        assert_eq!(
            vec![b'3', b'0', b'3', b'7', b'3'],
            iter.next().unwrap().map(|a| *a.1).collect::<Vec<u8>>()
        );

        // Second horizontal
        assert_eq!(
            vec![b'2', b'5', b'5', b'1', b'2'],
            iter.next().unwrap().map(|a| *a.1).collect::<Vec<u8>>()
        );

        // Skip the next 3 horizontals
        iter.next();
        iter.next();
        iter.next();

        // First horizontal from the right
        assert_eq!(
            vec![b'3', b'7', b'3', b'0', b'3'],
            iter.next().unwrap().map(|a| *a.1).collect::<Vec<u8>>()
        );

        // Second horizontal from the right
        assert_eq!(
            vec![b'2', b'1', b'5', b'5', b'2'],
            iter.next().unwrap().map(|a| *a.1).collect::<Vec<u8>>()
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

    fn run_nb_test(size: usize, pos: Vec2D<i32>) -> usize {
        let g = Grid {
            width: size,
            height: size,
            bytes: vec![1],
        };

        let mut vec: Vec<Vec2D<i32>> = Vec::new();
        g.get_neighbours_diagonal(pos, &mut vec);
        vec.len()
    }

    #[test]
    fn neigbours() {
        //1x1, should see nothing
        assert_eq!(run_nb_test(1, Vec2D { x: 0, y: 0 }), 0);

        // 3x3, center should have all 8
        assert_eq!(run_nb_test(3, Vec2D { x: 1, y: 1 }), 8);

        // 3x3, corners should see 3
        assert_eq!(run_nb_test(3, Vec2D { x: 0, y: 0 }), 3);
        assert_eq!(run_nb_test(3, Vec2D { x: 2, y: 0 }), 3);
        assert_eq!(run_nb_test(3, Vec2D { x: 0, y: 2 }), 3);
        assert_eq!(run_nb_test(3, Vec2D { x: 2, y: 2 }), 3);

        //3x3 center edges should see 5
        assert_eq!(run_nb_test(3, Vec2D { x: 1, y: 0 }), 5);
        assert_eq!(run_nb_test(3, Vec2D { x: 2, y: 1 }), 5);
        assert_eq!(run_nb_test(3, Vec2D { x: 1, y: 2 }), 5);
        assert_eq!(run_nb_test(3, Vec2D { x: 0, y: 1 }), 5);
    }
}
