use std::{
    fmt::Display,
    ops::{Add, Sub},
};

#[derive(Clone, PartialEq, Eq, Debug, Copy, Default, Hash)]
pub struct Vec2D<T> {
    pub x: T,
    pub y: T,
}

impl Sub for Vec2D<i32> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add for Vec2D<i32> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Vec2D<i32> {
    pub fn distance_manhatten(&self, b: &Self) -> i32 {
        (self.x - b.x).abs() + (self.y - b.y).abs()
    }

    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }
}

trait Bounds {
    fn bounds(&self) -> (Vec2D<i32>, Vec2D<i32>);
}

impl Bounds for [Vec2D<i32>] {
    fn bounds(&self) -> (Vec2D<i32>, Vec2D<i32>) {
        let mut min: Vec2D<i32> = *self.get(0).expect("At least one element");
        let mut max: Vec2D<i32> = min;

        for vec in self {
            if vec.x < min.x {
                min.x = vec.x
            }
            if vec.y < min.y {
                min.y = vec.y
            }

            if vec.x > max.x {
                max.x = vec.x
            }
            if vec.y > max.y {
                max.y = vec.y
            }
        }

        (min, max)
    }
}

pub fn print_vector_path(v: &[Vec2D<i32>]) {
    let (min, _) = v.bounds();
}

// impl Display for [Vec2D<i32>] {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }
